//  Copyright (c) Facebook, Inc. and its affiliates.
//  SPDX-License-Identifier: Apache-2.0

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let grpc_path = "proto";
    let proto_files = &[
        "common.proto",
        "pjc.proto",
    ];
    let out_env = if cfg!(fbcode_build) { "OUT" } else { "OUT_DIR" };
    let out_dir = std::env::var_os(out_env).unwrap_or_else(|| panic!("env `{out_env}` is not set"));

    tonic_build::configure()
        .type_attribute("common.Payload", "#[serde_with::serde_as]\n#[derive(serde::Serialize, serde::Deserialize)]")
        .field_attribute("common.Payload.payload", "#[serde_as(as = \"Vec<serde_with::base64::Base64>\")]")
        .type_attribute("pjc.Init", "#[derive(serde::Serialize, serde::Deserialize)]")
        .type_attribute("pjc.Commitment", "#[derive(serde::Serialize, serde::Deserialize)]")
        .type_attribute("pjc.Stats", "#[derive(serde::Serialize, serde::Deserialize)]")
        .field_attribute("pjc.Stats.encrypted_sums", "#[serde(alias = \"encryptedSums\")]")
        .field_attribute("pjc.Stats.intersection_size", "#[serde(alias = \"intersectionSize\")]")
        .type_attribute("pjc.EncryptedSum", "#[derive(serde::Serialize, serde::Deserialize)]")
        .out_dir(out_dir)
        .compile(
            proto_files,
            // HACK: we need '.' directory for build with Buck
            &[".", grpc_path],
        )
        .unwrap();

    Ok(())
}
