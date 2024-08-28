./proof-arena-integration/SPJ -cpu 64 -largestN 1365 -memory 32768 -time 1200 \
    -prover "proof-arena-integration/target/release/proof-arena-integration ./plonky3-keccak-serve/target/release/plonky3-keccak-serve prove 1365" \
    -verifier "proof-arena-integration/target/release/proof-arena-integration ./plonky3-keccak-serve/target/release/plonky3-keccak-serve verify 1000" \
    -json "result.json"
