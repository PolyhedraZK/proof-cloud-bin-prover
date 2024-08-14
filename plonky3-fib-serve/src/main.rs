use log::{debug, info};
use warp::{http::StatusCode, reply, Filter};

use p3_baby_bear::{BabyBear, DiffusionMatrixBabyBear};
use p3_fri::FriConfig;
use p3_poseidon2::Poseidon2ExternalMatrixGeneral;
use p3_uni_stark::{prove, verify};
use plonky3_fib_serve::air::*;

use std::sync::{Arc, Mutex};

use rand::thread_rng;

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
    // setup
    let perm = Perm::new_from_rng_128(
        Poseidon2ExternalMatrixGeneral,
        DiffusionMatrixBabyBear::default(),
        &mut thread_rng(),
    );
    let perm_prove = Arc::new(Mutex::new(perm.clone()));
    let perm_verify = Arc::new(Mutex::new(perm.clone()));
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
                    let perm_prove = perm_prove.lock().unwrap();
                    let hash = MyHash::new(perm_prove.clone());
                    let compress = MyCompress::new(perm_prove.clone());
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
                    let mut challenger = Challenger::new(perm_prove.clone());
                    // prove
                    let (trace, pis) = (witness.trace, witness.pis);
                    let proof = prove(&config, &FibonacciAir {}, &mut challenger, trace, &pis);
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

            let pis: Option<Vec<BabyBear>> = match postcard::from_bytes(pis_bytes) {
                Ok(pis) => {
                    debug!("Deserialized pis.");
                    Some(pis)
                }
                Err(e) => {
                    info!("Unable to deserialize pis: {:?}", e);
                    None
                }
            };
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
            if pis.is_none() || proof.is_none() {
                "failure".to_string()
            } else {
                // setup
                let perm_verify = perm_verify.lock().unwrap();
                let hash = MyHash::new(perm_verify.clone());
                let compress = MyCompress::new(perm_verify.clone());
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
                let mut challenger = Challenger::new(perm_verify.clone());
                let result = verify(
                    &config,
                    &FibonacciAir {},
                    &mut challenger,
                    &proof.unwrap(),
                    &pis.unwrap(),
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
