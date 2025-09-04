#!/bin/bash

EXTRA_OPTS=$1
TAG="fdeantoni/echo-server"

docker buildx build ${EXTRA_OPTS} --platform linux/amd64,linux/arm64/v8 -t ${TAG}:0.3.0 -t ${TAG}:latest -f docker/Dockerfile --push .
