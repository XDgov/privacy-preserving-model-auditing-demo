//  Copyright (c) Facebook, Inc. and its affiliates.
//  SPDX-License-Identifier: Apache-2.0

use base64; 
use clap::App;
use clap::Arg;
use clap::ArgGroup;
use common::timer;
use tonic::Request;
use tonic::transport::Body;
mod rpc_client;
use crypto::prelude::ByteBuffer;
use crypto::prelude::TPayload;
use log::info;
use protocol::pjc::partner::PartnerPjc;
use protocol::pjc::traits::*;
use protocol::shared::LoadData;
use protocol::shared::ShareableEncKey;
use prost::Message;
use reqwest; 
use rpc::connect::create_client::create_client;
use rpc::proto::common::Payload;
use rpc::proto::gen_pjc::service_response::*;
use rpc::proto::gen_pjc::Init;
use rpc::proto::gen_pjc::ServiceResponse;
use rpc::proto::gen_pjc::Stats;
use rpc::proto::RpcClient;
use serde_json;
use serde::{Serialize, Deserialize};
use std::fmt::Debug;

#[derive(Serialize, Deserialize, Debug)]
struct KeyResponse {
    result: Payload,
}

#[derive(Serialize, Deserialize, Debug)]
struct KeyPayload{
    payload: TPayload,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    // todo: move matches outside, or move to build.rs
    let matches = App::new("Join and compute clients")
        .version("0.1")
        .about("Cross PSI Protocol")
        .args(&[
            Arg::with_name("company")
                .long("company")
                .short("c")
                .takes_value(true)
                .required(true)
                .help("Host path to connect to, ex: 0.0.0.0:10011"),
            Arg::with_name("input")
                .long("input")
                .short("i")
                .default_value("input.csv")
                .help("Path to input file with keys"),
            Arg::with_name("output")
                .long("output")
                .short("o")
                .takes_value(true)
                .help("Path to output file, output format: private-id, option(key)"),
            Arg::with_name("stdout")
                .long("stdout")
                .short("u")
                .takes_value(false)
                .help("Prints the output to stdout rather than file"),
            Arg::with_name("no-tls")
                .long("no-tls")
                .takes_value(false)
                .help("Turns tls off"),
            Arg::with_name("tls-dir")
                .long("tls-dir")
                .takes_value(true)
                .help(
                    "Path to directory with files with key, cert and ca.pem file\n
                    client: client.key, client.pem, ca.pem \n
                    server: server.key, server.pem, ca.pem \n
                ",
                ),
            Arg::with_name("tls-key")
                .long("tls-key")
                .takes_value(true)
                .requires("tls-cert")
                .requires("tls-ca")
                .help("Path to tls key (non-encrypted)"),
            Arg::with_name("tls-cert")
                .long("tls-cert")
                .takes_value(true)
                .requires("tls-key")
                .requires("tls-ca")
                .help(
                    "Path to tls certificate (pem format), SINGLE cert, \
                     NO CHAINING, required by client as well",
                ),
            Arg::with_name("tls-ca")
                .long("tls-ca")
                .takes_value(true)
                .help("Path to root CA certificate issued cert and keys"),
            Arg::with_name("tls-domain")
                .long("tls-domain")
                .takes_value(true)
                .help("Override TLS domain for SSL cert (if host is IP)"),
        ])
        .groups(&[
            ArgGroup::with_name("tls")
                .args(&["no-tls", "tls-dir", "tls-ca"])
                .required(true),
            ArgGroup::with_name("out")
                .args(&["output", "stdout"])
                .required(true),
        ])
        .get_matches();

    let global_timer = timer::Timer::new_silent("global");
    let input_path = matches.value_of("input").unwrap_or("input.csv");
    // let output_path = matches.value_of("output");

    /*let mut client_context = {
        let no_tls = matches.is_present("no-tls");
        let host_pre = matches.value_of("company");
        let tls_dir = matches.value_of("tls-dir");
        let tls_key = matches.value_of("tls-key");
        let tls_cert = matches.value_of("tls-cert");
        let tls_ca = matches.value_of("tls-ca");
        let tls_domain = matches.value_of("tls-domain");

        match create_client(
            no_tls,
            host_pre,
            tls_dir,
            tls_key,
            tls_cert,
            tls_ca,
            tls_domain,
            "pjc".to_string(),
        ) {
            RpcClient::Pjc(x) => x,
            _ => panic!("wrong client"),
        }
    };*/

    let partner_protocol = PartnerPjc::new();


    let host_pre = matches.value_of("company");
    // 1. Load data
    // 2. Fill permutation pattern
    partner_protocol.load_data(input_path);
    partner_protocol.fill_permute_self();

    // 3. Send public key for Homomorphic encryption to company
    info!("Sending HE key to company");
    let key = partner_protocol.get_he_public_key();
    let payload = Init{
        public_key: Some(Payload::from(&partner_protocol.get_he_public_key()))
    };

    let json_pl = serde_json::json!(&payload);
    let http_client = reqwest::Client::new();
    let init_ack = http_client.post(
        format!("{}/v1/key_exchange", &host_pre.unwrap())
    ).json(&json_pl)
    .send()
    .await?;

    info!("Receiving key from company");
    // 4. Receive encrypted keys from company

    let resp = http_client.post(
        format!("{}/v1/recv_u_company_keys", &host_pre.unwrap())
    ).send().await?.json::<KeyResponse>().await?;

    let byte_array : Vec<ByteBuffer> = resp.result.payload.iter().map(|e| ByteBuffer{buffer: e.to_vec()}).collect();

    let mut u_company_keys = TPayload::from(byte_array);

    println!("{:?}", u_company_keys);


    info!("encrypting and permuting");
    // 5. Encrypt company's keys with own keys and permute
    let e_company_keys = partner_protocol.encrypt_permute(u_company_keys);

    let resp = rpc_client::send_rest_api(
        e_company_keys,
        "e_company_keys".to_string(),
        &http_client,
        host_pre.unwrap().to_string()
    ).await;
    
    println!("{:?}", resp);


    // 7. Send partner's permuted and encrypted keys to company to calculate
    //    intersection

    let u_partner_keys = partner_protocol.get_permuted_keys();

    let resp = rpc_client::send_rest_api(
        u_partner_keys,
        "u_partner_keys".to_string(),
        &http_client,
        host_pre.unwrap().to_string()
    ).await;

    println!("{:?}", resp);


    // 8. Send partner's permuted and encrypted features to company to calculate
    //    encrypted sum for each feature. This sums values that correspond to keys
    //    that are common to both partner and company
    for feature_index in 0..partner_protocol.get_self_num_features() {
        let mut feature = partner_protocol.get_permuted_features(feature_index);
        feature.push(ByteBuffer {
            buffer: (feature_index as u64).to_le_bytes().to_vec(),
        });

        let resp = rpc_client::send_rest_api(
            feature,
            "u_partner_feature".to_string(),
            &http_client,
            host_pre.unwrap().to_string()
        ).await;
    
        println!("{:?}", resp);
    }

    // 9. Receive sums of each feature
    let proto_stats = http_client.post(
        format!("{}/v1/recv_stats", &host_pre.unwrap())
    ).send().await?.json::<Stats>().await?;

    let encrypted_sums = proto_stats
        .encrypted_sums
        .into_iter()
        .map(|result| TPayload::from(&result.payload.unwrap()))
        .collect::<Vec<TPayload>>();

    println!("vector length {}", encrypted_sums.len());

    // 10. Decrypt sums
    partner_protocol.decrypt_stats(encrypted_sums);

    global_timer.qps(
        "total time",
        partner_protocol.get_self_num_features() * partner_protocol.get_self_num_records(),
    );

    Ok(())
}
