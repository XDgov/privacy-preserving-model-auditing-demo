# Census xD demo of Private Join and Compute protocol on cloud.gov

This repository is a demonstration of how to get an open source implementation of a cryptographic Privacy Enhancing Technologies protocol, Private Join and Compute (PJC), running on cloud.gov infrastructure. It is an adaptation of Facebook Research's [Private-ID](https://github.com/facebookresearch/Private-ID) repository. While that repository contained several different algorithms for record matching, this demo focuses on PJC, which is a private set intersection sum with cardinality protocol from [Google](https://security.googleblog.com/2019/06/helping-organizations-do-more-without-collecting-more-data.html).
 

## Requirements

- [Rust](https://www.rust-lang.org/tools/install)
- [Go](https://go.dev/doc/install)
- [Cross](https://github.com/cross-rs/cross)
  - [Podman](https://podman.io/docs/installation) (dependency of Cross)
- [Cloud Foundry CLI](https://www.cloudfoundry.org/) v8
- [buf](https://buf.build/docs/installation)

You will also need a [cloud.gov](https://cloud.gov/docs/getting-started/your-first-deploy/) account, organization, and space. 

## Simple demo

The Private-ID package implements a server and client for the PJC protocol in Rust. This demo deploys the "company" server onto the federal [cloud.gov](cloud.gov) infrastructure, while the "partner" client runs locally. (In our demonstration, we ran the local client on a Mac OS X machine). Because we ran into some memory limits while attempting to build directly on cloud.gov, we instead cross-compile binaries from Mac OS X to Linux locally and use cloud.gov's [binary buildpack](https://docs.cloudfoundry.org/buildpacks/binary/index.html) for deployment. While the original implementation uses gRPC for communication, cloud.gov does not support this protocol, so we use the [grpc-gateway](https://grpc-ecosystem.github.io/grpc-gateway/) package to generate code for a standard REST API server that proxies calls to the RPC server. So, in order to get the demo running, you will have to take the following steps:

1. Cross-compile the Rust server binary using the [Cross](https://github.com/cross-rs/cross) package.
2. Generate Go proxy stubs from an annotated protobuf file in this repo. 
3. Cross-compile the Go proxy binary using the Go compiler. 
4. Deploy the proxy and RPC servers to cloud.gov
5. Compile and run the client locally. 

Each of these steps is described in more detail below. 

### Cross-compile Rust server binary

To use Cross, you first need a Docker or Podman machine running. 

```
podman machine init
podman machine start
```

To target Linux using cross, run the following command:

```
cross build --release --target x86_64-unknown-linux-gnu
```

NOTE: If you are behind a firewall, you may have an issue with SSL verification when the container for compilation is being fetched. If the error looks like this:

```
Error: creating build container: initializing source docker://ghcr.io/cross-rs/x86_64-unknown-linux-gnu:main: pinging container registry ghcr.io: Get "https://ghcr.io/v2/": tls: failed to verify certificate: x509: certificate signed by unknown authority
```

If you see this, fetch the container first with SSL verification turned off with the following command:

```
podman pull --platform=linux/x86_64 --tls-verify=false docker://ghcr.io/cross-rs/x86_64-unknown-linux-gnu:main
```

Now when you run the build command, it should work. 

```
cross build --release --target x86_64-unknown-linux-gnu
```

### Generate Go proxy server code and cross-compile proxy binary

To compile the Go binary, we first need to create a Go module in the appropriate directory. Then, we generate the proxy code from an annotated protobuf using `buf`. Then, finally, we build the binary. From the main repo directory, run

NOTE: you will need to modify the default argument in `pjc-proxy-go/main.go` to be the same as what you set as `$RPC_PREFIX` in the next step below or modify Procfile-proxy to pass the appropriate argument. 

```
cd pjc-proxy-go

# Initialize Go module
go mod init proxy-server
go mod tidy
 
# Generate proxy code
buf dep update
buf generate

# Cross-compile binary
GOOS=linux GOARCH=amd64 go build
```

Note that the `buf generate` step also generates a `swagger.json` API specification in the `openapiv2` directory. The full path is `openapiv2/proto/pjc-proxy/pjc.swagger.json`. 

### Deploy to cloud.gov

First follow the instructions [here](https://cloud.gov/docs/getting-started/your-first-deploy/) to set up your login credentials and target your organization's space.

Next, define two environment variables, `PROXY_PREFIX` and `RPC_PREFIX` with the prefixes you would like to have respectively for your proxy and RPC server URLs and app names. 

NOTE: The RPC server script is currently configured to use the example data file `etc/example/pjc_company.csv`. If you wish to change this, you will need to change the `.sh` file and also `Procfile-rpc` to point to the correct file. 

Then, from the root directory of the repository, deploy the RPC server with 

```
./push-rpc-server.sh
```

Once this is successfully running, deploy the proxy.

```
./push-proxy-server.sh
```

### Compile and run client

Our final step is to run the client, which will actually do the computation! To run the client, we run 

```
env RUST_LOG=info cargo run --release --bin pjc-client -- --company https://$PROXY_PREFIX.app.cloud.gov --input etc/example/pjc_partner.csv --stdout --no-tls
```

Note that you will have to replace the `--company` argument above with the URL of your proxy server's app. 

## Demographic disparity demo

The above was a simply demo using the files that came with the original Private-ID repository. If you would like to run an example of measuring model performance across demographic groups, as described [here](https://www.xd.gov/blog/privacy-preserving-model-auditing/), you can run the following scripts from the repo's root directory. 

```
./push-demographic-example.sh
```

will stand up three proxy and RPC servers per demographic group. Then, 

```
./run-demographic-client.sh
```

will run the PJC protocol against each of the servers and collate the results.

## Caveats

Because this is a demo, there is currently a lack of support for many things that a production API would have. These include:

- No API authentication
- No support for multiple clients

