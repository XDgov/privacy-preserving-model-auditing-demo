#!/bin/bash
set -e

PUSH_DIR=push_dir_proxy

mkdir -p $PUSH_DIR
cp manifest-proxy.yml ./$PUSH_DIR/manifest.yml
cp Procfile-proxy ./$PUSH_DIR/Procfile
cd pjc-proxy-go
GOOS=linux GOARCH=amd64 go build
cd ..
cp ./pjc-proxy-go/proxy-server ./$PUSH_DIR

cd ./$PUSH_DIR

cf push $PROXY_PREFIX
cf add-network-policy $PROXY_PREFIX $RPC_PREFIX -s dev -o census-xd-pets-prototyping --protocol tcp --port 8080