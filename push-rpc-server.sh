#!/bin/bash

VARS_FILE=binary-buildpack-vars.yml
PUSH_DIR=push_dir_rpc

mkdir -p $PUSH_DIR
cp ./target/x86_64-unknown-linux-gnu/release/pjc-server ./$PUSH_DIR
cp manifest-rpc.yml ./$PUSH_DIR/manifest.yml
cp Procfile-rpc ./$PUSH_DIR/Procfile
cp $VARS_FILE ./$PUSH_DIR
cp etc/example/pjc_company.csv ./$PUSH_DIR

cd ./$PUSH_DIR

cf push --vars-file $VARS_FILE test-lazovich-binary-pjc
cf map-route test-lazovich-binary-pjc apps.internal --hostname test-lazovich-binary-pjc --app-protocol http2