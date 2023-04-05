FROM ubuntu:latest

RUN apt update && \
    apt install -y curl && \
    apt install -y curl build-essential && \
    apt install -y protobuf-compiler libprotobuf-dev && \
    apt install -y liburing-dev

RUN mkdir -p /home/ministore
WORKDIR "/home/ministore"

RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
ENV PATH="/root/.cargo/bin:${PATH}"

COPY . .

# Will change to run program when I'm ready!
#RUN cargo build --release
#CMD ["./target/release/ministore"]
