#!/bin/bash
set -e

PUSH_DIR=push_dir_rpc

mkdir -p $PUSH_DIR
cp ./target/x86_64-unknown-linux-gnu/release/pjc-server ./$PUSH_DIR
cp manifest-rpc.yml ./$PUSH_DIR/manifest.yml
cp Procfile-rpc ./$PUSH_DIR/Procfile
cp etc/example/pjc_company.csv ./$PUSH_DIR

cd ./$PUSH_DIR

cf push $RPC_PREFIX
cf map-route $RPC_PREFIX apps.internal --hostname $RPC_PREFIX --app-protocol http2