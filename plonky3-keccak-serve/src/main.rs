use log::{debug, info};
use p3_keccak::Keccak256Hash;
use p3_keccak_air::{generate_trace_rows, KeccakAir};
use warp::{http::StatusCode, reply, Filter};

use p3_fri::FriConfig;
use p3_uni_stark::{prove, verify, StarkConfig};
use plonky3_keccak_serve::*;

use std::sync::{Arc, Mutex};

#[tokio::main]
async fn main() {
    env_logger::init();
    // plonky3-fib-serve <input:ip> <input:port>
    // parse arg
    let args = std::env::args().collect::<Vec<String>>();
    if args.len() != 3 {
        println!("Usage: plonky3-fib-serve serve <input:host> <input:port>");
        return;
    }
    let host: [u8; 4] = args[1]
        .split('.')
        .map(|s| s.parse().unwrap())
        .collect::<Vec<u8>>()
        .try_into()
        .unwrap();
    let port = args[2].parse().unwrap();

    // copied above

    // let perm = Perm::new_from_rng_128(
    //     Poseidon2ExternalMatrixGeneral,
    //     DiffusionMatrixBabyBear::default(),
    //     &mut thread_rng(),
    // );
    let ready_time = chrono::offset::Utc::now();

    // endpoints
    let ready = warp::path("ready").map(move || {
        info!("Received ready request.");
        reply::with_status(format!("Ready since {:?}", ready_time), StatusCode::OK)
    });
    let prove = warp::path("prove")
        .and(warp::body::bytes())
        .map(move |bytes: bytes::Bytes| {
            info!("Received prove request.");
            let witness_bytes: Vec<u8> = bytes.to_vec();
            match postcard::from_bytes::<MyWitness>(&witness_bytes) {
                Ok(witness) => {
                    debug!("Deserialized witness.");
                    // setup
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
                    // prove
                    let mut challenger = Challenger::from_hasher(vec![], byte_hash);
                    let trace = generate_trace_rows::<Val>(witness.inputs);
                    let proof = prove(&config, &KeccakAir {}, &mut challenger, trace, &vec![]);
                    let proof_serialized =
                        postcard::to_allocvec(&proof).expect("unable to serialize proof");
                    reply::with_status(proof_serialized, StatusCode::OK)
                }
                Err(e) => {
                    info!("Unable to deserialize witness: {:?}", e);
                    reply::with_status(vec![], StatusCode::BAD_REQUEST)
                }
            }
        });
    let verify = warp::path("verify")
        .and(warp::body::bytes())
        .map(move |bytes: bytes::Bytes| {
            info!("Received verify request.");
            let pis_and_proof_bytes: Vec<u8> = bytes.to_vec();
            let length_of_pis_bytes =
                u64::from_le_bytes(pis_and_proof_bytes[0..8].try_into().unwrap()) as usize;
            let length_of_proof_bytes =
                u64::from_le_bytes(pis_and_proof_bytes[8..16].try_into().unwrap()) as usize;
            let pis_bytes = &pis_and_proof_bytes[16..16 + length_of_pis_bytes];
            let proof_bytes = &pis_and_proof_bytes
                [16 + length_of_pis_bytes..16 + length_of_pis_bytes + length_of_proof_bytes];
            // keccak do not need pis
            let proof = match postcard::from_bytes(proof_bytes) {
                Ok(proof) => {
                    debug!("Deserialized proof.");
                    Some(proof)
                }
                Err(e) => {
                    info!("Unable to deserialize proof: {:?}", e);
                    None
                }
            };

            if proof.is_none() {
                "failure".to_string()
            } else {
                // setup
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
                // prove
                let mut challenger = Challenger::from_hasher(vec![], byte_hash);
                let result = verify(
                    &config,
                    &KeccakAir {},
                    &mut challenger,
                    &proof.unwrap(),
                    &vec![],
                );
                if result.is_ok() {
                    "success".to_string()
                } else {
                    "failure".to_string()
                }
            }
        });
    warp::serve(
        warp::post()
            .and(prove.or(verify))
            .or(warp::get().and(ready)),
    )
    .run((host, port))
    .await;
}
