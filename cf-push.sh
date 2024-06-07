#!/bin/bash

VARS_FILE=binary-buildpack-vars.yml

mkdir -p push_dir
cp ./target/x86_64-unknown-linux-gnu/release/pjc-server ./push_dir
cp manifest.yml ./push_dir
cp Procfile ./push_dir
cp $VARS_FILE ./push_dir
cp etc/example/pjc_company.csv ./push_dir

cd ./push_dir

cf push --vars-file $VARS_FILE test-lazovich-binary-pjc