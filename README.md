# Service

## Testing

```
git clone git@github.com:ProvableHQ/snarkOS.git
cd snarkOS
./devnet.sh (use network ID 2)
```

```
git clone git@github.com:ProvableHQ/service.git
cargo build --release
./target/release/authorize-service --network canary
./target/release/execute-service --network canary
./target/release/transfer_client --generate-requests
```

