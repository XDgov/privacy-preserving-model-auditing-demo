package main

import (
  "context"
  "flag"
  "net/http"
  "fmt"
  "github.com/grpc-ecosystem/grpc-gateway/v2/runtime"
  "google.golang.org/grpc"
  "google.golang.org/grpc/credentials/insecure"
  "google.golang.org/grpc/grpclog"

  gw "proxy-server/proto/pjc-proxy"  // Update
)

var (
  // command-line options:
  // gRPC server endpoint
  // Change this default argument to your server name or otherwise change Procfile-proxy to pass the appropriate argument
  grpcServerEndpoint = flag.String("grpc-server-endpoint",  "$RPC_PREFIX.apps.internal:8080", "gRPC server endpoint")
)

func run() error {
  ctx := context.Background()
  ctx, cancel := context.WithCancel(ctx)
  defer cancel()

  // Register gRPC server endpoint
  // Note: Make sure the gRPC server is running properly and accessible
  mux := runtime.NewServeMux()
  opts := []grpc.DialOption{grpc.WithTransportCredentials(insecure.NewCredentials())}
  err := gw.RegisterPJCHandlerFromEndpoint(ctx, mux,  *grpcServerEndpoint, opts)
  if err != nil {
    return err
  }

  // Start HTTP server (and proxy calls to gRPC server endpoint)
  return http.ListenAndServe(":8080", mux)
}

func main() {
  flag.Parse()

  fmt.Print("Starting proxy server!")
  if err := run(); err != nil {
    fmt.Print("Welcome to errorland, population you!")
    grpclog.Fatal(err)
  }
}
