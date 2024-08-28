./proof-arena-integration/SPJ -cpu 64 -largestN 8 -memory 32768 \
    -prover "proof-arena-integration/target/release/proof-arena-integration ./plonky3-keccak-serve/target/release/plonky3-keccak-serve prove" \
    -time 300 \
    -verifier "proof-arena-integration/target/release/proof-arena-integration ./plonky3-keccak-serve/target/release/plonky3-keccak-serve verify" \
    -json "result.json"
