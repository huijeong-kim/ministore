#!/bin/bash

test=""

build_image=0

while [[ "$#" -gt 0 ]]; do
    case $1 in
        -n|--no-build) build_image=0 ;;
        -b|--build) build_image=1 ;;
        -t|--test) test="$2"; shift ;;
        -h|--help) echo "Usage: $(dirname "$0") [testname]"
                    echo "   Build docker image and run test"
                    echo "   -n, --no-build: do not build a docker image (default)"
                    echo "   -b, --build: build a docker image before start test"
                    echo "   -t, --test: specify a test to run (default: run all tests)"
                    echo "   -h, --help: print this help message"
                    exit 0 ;;
        *) echo "Invalid option: $1" 1>&2
            exit 1 ;;
    esac
    shift
done 

working_dir="$(readlink -f "$(dirname "$0")")/.."

if [ "$build_image" -eq 1 ]; then
    # Build docker image
    echo ">>> Build docker image"
    docker build -t ministore ${working_dir}
fi 

# Run docker image and execute 'cargo test'
echo ">>> Run cargo test"
# Working directory in the container is "/home/ministore". Please refer to the dockerfile
docker run -v ${working_dir}:/home/ministore ministore cargo test ${test}