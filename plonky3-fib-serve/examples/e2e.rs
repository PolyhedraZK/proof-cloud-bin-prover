use p3_baby_bear::{BabyBear, DiffusionMatrixBabyBear};
use p3_field::AbstractField;
use p3_fri::FriConfig;
use p3_poseidon2::Poseidon2ExternalMatrixGeneral;
use p3_uni_stark::{prove, verify};
use plonky3_fib_serve::air::*;

use rand::thread_rng;

const WITNESS_LOC: &str = "../example_witness.bin";
const PIS_LOC: &str = "../example_pis.bin";
const PROOF_LOC: &str = "../example_proof.bin";

fn main() {
    // circuit-agnostic setup
    let perm = Perm::new_from_rng_128(
        Poseidon2ExternalMatrixGeneral,
        DiffusionMatrixBabyBear::default(),
        &mut thread_rng(),
    );
    let hash = MyHash::new(perm.clone());
    let compress = MyCompress::new(perm.clone());
    let val_mmcs = ValMmcs::new(hash, compress);
    let challenge_mmcs = ChallengeMmcs::new(val_mmcs.clone());
    let dft = Dft {};
    let fri_config = FriConfig {
        log_blowup: 2,
        num_queries: 28,
        proof_of_work_bits: 8,
        mmcs: challenge_mmcs,
    };
    let pcs = Pcs::new(dft, val_mmcs, fri_config);
    let config = MyConfig::new(pcs);
    let mut challenger = Challenger::new(perm.clone());

    // execution trace
    let trace = generate_trace_rows::<Val>(0, 1, 1 << 3);
    println!("trace (2X8): {:?}", trace);
    let pis = vec![
        BabyBear::from_canonical_u64(0),
        BabyBear::from_canonical_u64(1),
        BabyBear::from_canonical_u64(21),
    ];

    // witness: trace and public value
    let pis_serialized = postcard::to_allocvec(&pis.clone()).expect("unable to serialize pis");
    std::fs::write(PIS_LOC, &pis_serialized).expect("unable to write pis to file");
    let witness = MyWitness { trace, pis };
    let witness_serialized = postcard::to_allocvec(&witness).expect("unable to serialize witness");
    std::fs::write(WITNESS_LOC, &witness_serialized).expect("unable to write witness to file");
    let witness: MyWitness =
        postcard::from_bytes(&witness_serialized).expect("unable to deserialize witness");
    let (trace, pis) = (witness.trace, witness.pis);

    let proof = prove(&config, &FibonacciAir {}, &mut challenger, trace, &pis);
    let proof_serialized = postcard::to_allocvec(&proof).expect("unable to serialize proof");
    std::fs::write(PROOF_LOC, &proof_serialized).expect("unable to write proof to file");

    let mut challenger = Challenger::new(perm);
    verify(&config, &FibonacciAir {}, &mut challenger, &proof, &pis).expect("verification failed");
    println!("verification successful");
}
