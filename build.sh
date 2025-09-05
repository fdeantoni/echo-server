#!/bin/bash
shopt -s nocasematch

get_version() {
  local version=`cargo metadata --format-version 1 | jq -r '.packages[] | select( .name == "echo-server" ) | .version ' | tr '[:upper:]' '[:lower:]'`
  echo $version
}

EXTRA_OPTS=$1
TAG="fdeantoni/echo-server"

docker buildx build ${EXTRA_OPTS} --platform linux/amd64,linux/arm64/v8 -t ${TAG}:$(get_version) -t ${TAG}:latest -f docker/Dockerfile --push .
