// Modified from: https://github.com/Plonky3/Plonky3/blob/55832146c86e8e4d246bb9843da17f2159d212a5/keccak-air/examples/prove_baby_bear_keccak.rs

use p3_fri::FriConfig;
use p3_keccak::Keccak256Hash;
use p3_keccak_air::{generate_trace_rows, KeccakAir};

use p3_uni_stark::{prove, verify};
use plonky3_keccak_serve::*;
use rand::random;

const WITNESS_LOC: &str = "../example_witness.bin";
const PIS_LOC: &str = "../example_pis.bin";
const PROOF_LOC: &str = "../example_proof.bin";

fn main() {
    let byte_hash = ByteHash {};
    let field_hash = FieldHash::new(Keccak256Hash {});
    let compress = MyCompress::new(byte_hash);
    let val_mmcs = ValMmcs::new(field_hash, compress);
    let challenge_mmcs = ChallengeMmcs::new(val_mmcs.clone());
    let dft = Dft {};
    let fri_config = FriConfig {
        log_blowup: 1,
        num_queries: 100,
        proof_of_work_bits: 16,
        mmcs: challenge_mmcs,
    };
    let pcs = Pcs::new(dft, val_mmcs, fri_config);
    let config = MyConfig::new(pcs);

    // random input
    let inputs = (0..NUM_HASHES).map(|_| random()).collect::<Vec<_>>();
    let pis = &inputs;

    // witness: trace and public value
    let pis_serialized = postcard::to_allocvec(&pis.clone()).expect("unable to serialize pis");
    std::fs::write(PIS_LOC, &pis_serialized).expect("unable to write pis to file");
    let witness = MyWitness { inputs };
    let witness_serialized = postcard::to_allocvec(&witness).expect("unable to serialize witness");
    std::fs::write(WITNESS_LOC, &witness_serialized).expect("unable to write witness to file");
    let witness: MyWitness =
        postcard::from_bytes(&witness_serialized).expect("unable to deserialize witness");
    let inputs = witness.inputs;

    let mut challenger = Challenger::from_hasher(vec![], byte_hash);
    let trace = generate_trace_rows::<Val>(inputs.clone());
    let proof = prove(&config, &KeccakAir {}, &mut challenger, trace, &vec![]);
    let proof_serialized = postcard::to_allocvec(&proof).expect("unable to serialize proof");
    std::fs::write(PROOF_LOC, &proof_serialized).expect("unable to write proof to file");

    let mut challenger = Challenger::from_hasher(vec![], byte_hash);
    verify(&config, &KeccakAir {}, &mut challenger, &proof, &vec![]).expect("verification failed");
    println!("verification successful");
}
