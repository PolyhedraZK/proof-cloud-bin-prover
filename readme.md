# Proof cloud - bin prover integration examples

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
