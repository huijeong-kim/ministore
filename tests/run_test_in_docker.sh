#!/bin/bash

test=""

if [[ "$1" == "-h" || "$1" == "--help" ]]; then
    echo "Usage: $0 [testname]"
    echo "   Build docker image and run test"
    echo "   If no testname is specified, run all tests"
    exit 0
elif [ -n "$1" ]; then 
    test="$1"
fi

working_dir="$(readlink -f "$(dirname "$0")")/.."

# Build docker image
echo ">>> Build docker image"
docker build -t ministore ${working_dir}

# Run docker image and execute 'cargo test'
echo ">>> Run cargo test"
docker run --rm ministore sh -c "cargo test ${test}"
