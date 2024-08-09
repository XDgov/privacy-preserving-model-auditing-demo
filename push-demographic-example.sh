#!/bin/bash
set -e

declare -a demo_groups=("White" "Asian" "Alaska_Native")


for grp in "${demo_groups[@]}"
do
    echo "==============Deploying for demographic group ${grp}=================="

    PUSH_DIR=push_dir_rpc_$grp

    mkdir -p $PUSH_DIR
    cp ./target/x86_64-unknown-linux-gnu/release/pjc-server ./$PUSH_DIR
    cp manifest-rpc.yml ./$PUSH_DIR/manifest.yml
    cp Procfile-rpc ./$PUSH_DIR/Procfile
    cp etc/example/demographic_info_$grp.csv ./$PUSH_DIR/pjc_company.csv

    cd ./$PUSH_DIR

    app_name_rpc=$RPC_PREFIX-$grp

    cf push $app_name_rpc
    cf map-route $app_name_rpc apps.internal --hostname $app_name_rpc --app-protocol http2

    cd ..

    PUSH_DIR=push_dir_proxy_$grp

    mkdir -p $PUSH_DIR
    cp manifest-proxy.yml ./$PUSH_DIR/manifest.yml
    cp Procfile-proxy ./$PUSH_DIR/Procfile
    cd pjc-proxy-go
    GOOS=linux GOARCH=amd64 go build
    cd ..
    cp ./pjc-proxy-go/proxy-server ./$PUSH_DIR

    cd ./$PUSH_DIR

    app_name_proxy=$PROXY_PREFIX-$grp
    echo " --grpc-server-endpoint $app_name_rpc.apps.internal:8080" >> Procfile
    cf push $app_name_proxy
    cf add-network-policy $app_name_proxy $app_name_rpc -s dev -o census-xd-pets-prototyping --protocol tcp --port 8080

    cd ..
done



