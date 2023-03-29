use std::net::SocketAddr;
use tonic::transport::Server;
use tonic::Response;

use self::ministore_proto::mini_service_server::{MiniService, MiniServiceServer};
use self::ministore_proto::{
    CreateFakeDeviceRequest, CreateFakeDeviceResponse, Status, StatusRequest, StatusResponse,
};

pub mod ministore_proto {
    tonic::include_proto!("ministore");
}

pub async fn start_grpc_server(addr: SocketAddr) -> Result<(), String> {
    Server::builder()
        .add_service(MiniServiceServer::new(GrpcServer::default()))
        .serve(addr)
        .await
        .map_err(|e| e.to_string())?;

    Ok(())
}

#[derive(Default)]
pub struct GrpcServer {}

#[tonic::async_trait]
impl MiniService for GrpcServer {
    async fn status(
        &self,
        _request: tonic::Request<StatusRequest>,
    ) -> Result<tonic::Response<StatusResponse>, tonic::Status> {
        let reply = StatusResponse {
            status: Status::Ready as i32,
        };

        Ok(Response::new(reply))
    }

    async fn create_fake_device(
        &self,
        request: tonic::Request<CreateFakeDeviceRequest>,
    ) -> Result<tonic::Response<CreateFakeDeviceResponse>, tonic::Status> {
        let _request = request.into_inner();

        let reply = CreateFakeDeviceResponse {
            success: true,
            reason: None,
        };

        Ok(Response::new(reply))
    }
}

#[cfg(test)]
mod tests {
    use crate::grpc_server::ministore_proto::mini_service_client::MiniServiceClient;

    use super::*;

    #[tokio::test]
    async fn server_should_response_with_ready_when_started() {
        let addr = "127.0.0.1:8080";
        let addr_for_server = addr.parse().expect("Failed to parse socket addr");
        let addr_for_client = format!("http://{}", addr.clone());

        tokio::spawn(async move {
            start_grpc_server(addr_for_server)
                .await
                .expect("Failed to start grpc server");
        });

        tokio::spawn(async move {
            let mut client = MiniServiceClient::connect(addr_for_client)
                .await
                .expect("Failed to start test client");
            let request = tonic::Request::new(StatusRequest {});

            let response = client
                .status(request)
                .await
                .expect("Failed to get response");
            assert_eq!(response.into_inner().status, Status::Ready as i32);
        });
    }
}
