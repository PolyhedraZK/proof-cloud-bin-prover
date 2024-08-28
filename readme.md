# Proof cloud - bin prover integration examples

A compatible prover is expected to expose the following endpoints:
- `GET /ready`: A proof-agnostic health check endpoint that returns 200 OK once the prover is ready to accept requests.
- `POST /prove`: Accept a byte array payload that contains serialized witness, returns a byte array payload that contains serialized proof.
- `POST /verify`: Accept a byte array payload that contains serialized public inputs and proof. Specifically, the first 8 bytes describe the length of the public inputs in little endian, then the next 8 bytes describe the length of the proof also in little endian, and the rest of the payload contains the public inputs and proof with the given lengths. Return a string "success" or "failure + <optional err message>" based on the verification result. 

Additionally, the host IP and port should be configurable via command line arguments. If the prover support customized circuits or other custom flags, they should be configurable via command line arguments as well. 

## Plonky3 (keccak example)

- To run keccak example and generate `example_witness.bin` and `example_proof.bin` files:

```sh
cd plonky3-keccak-serve
RUSTFLAGS="-Ctarget-cpu=native" cargo run --example e2e
```

- Run prover&verifier service

```sh
cd plonky3-keccak-serve
RUSTFLAGS="-Ctarget-cpu=native" cargo run -- 127.0.0.1 3030
```

- To test the service

```sh
python3 ./scripts/test_http.py  # need "requests" package, note that this may take a while
```


## Plonky3 (fib example)

- To run fib example and generate `example_witness.bin` and `example_proof.bin` files:

```sh
cd plonky3-fib-serve
cargo run --example e2e
```

- Run prover&verifier service

```sh
cd plonky3-fib-serve
cargo run -- 127.0.0.1 3030
```

- To test the service

```sh
python3 ./scripts/test_http.py  # need "requests" package
```


## Build plonky3 keccak for integration

Requirements: SPJ binary at `proof-arena-integration/SPJ`, you can find it from [proof arena repo](https://github.com/PolyhedraZK/proof-arena).

- Build service with release mode

```sh
cd plonky3-keccak-serve
RUSTFLAGS="-Ctarget-cpu=native" cargo build --release
```

- Build integration code

```sh
cd proof-arena-integration
RUSTFLAGS="-Ctarget-cpu=native" cargo build --release
```

- Run SPJ

```sh
./proof-arena-integration/run.sh
```
