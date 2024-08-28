use reqwest;
use std::{
    fs::File,
    io::{self, BufReader, Read, Write},
};
use tiny_keccak::Hasher;

const KECCAK_INSTANCE_NUM: usize = 8; // TODO LATER: change to 1365 along with the impl
const WITNESS_GENERATED_MSG: &str = "witness generated";
const VERIFIER_REPEAT_NUM: usize = 10;
const BUFFER_SIZE: usize = 128 * 1024 * 1024;
struct ServiceHandler {
    child: std::process::Child,
    port: u16,
}

fn start_service(service_bin: &str) -> ServiceHandler {
    // generate random port for the service
    let port = rand::random::<u16>();
    // run the service binary
    let child = std::process::Command::new(service_bin)
        .arg("127.0.0.1".to_string())
        .arg(port.to_string())
        .spawn()
        .expect("Failed to start the service");
    ServiceHandler { child, port }
}

impl ServiceHandler {
    fn prove(&mut self, inputs: &[u8]) -> Vec<u8> {
        // connect to the service via http request
        let client = reqwest::blocking::Client::new();
        let res = client
            .post(&format!("http://127.0.0.1:{}/prove", self.port))
            .body(inputs.to_vec())
            .send()
            .expect("Failed to send request");
        res.bytes().expect("Failed to read response").to_vec()
    }
    fn verify(&mut self, inputs: &[u8], proof: &[u8]) -> bool {
        // connect to the service via http request
        let client = reqwest::blocking::Client::new();
        let mut body = Vec::new();
        body.extend_from_slice(&(inputs.len() as u64).to_le_bytes());
        body.extend_from_slice(&(proof.len() as u64).to_le_bytes());
        body.extend_from_slice(inputs);
        body.extend_from_slice(proof);
        let res = client
            .post(&format!("http://127.0.0.1:{}/verify", self.port))
            .body(body)
            .send()
            .expect("Failed to send request");
        let res_str = res.text().expect("Failed to read response");
        res_str == "success"
    }
    fn stop(&mut self) {
        self.child.kill().expect("Failed to kill the service");
    }
}

fn baseline_hasher(inputs: &[u8]) -> Vec<u8> {
    // chunk the inputs into 64-byte blocks
    inputs
        .chunks_exact(64)
        .map(|x| {
            let mut hasher = tiny_keccak::Keccak::v256();
            let mut output = [0u8; 32];
            hasher.update(x);
            hasher.finalize(&mut output);
            output
        })
        .flatten()
        .collect()
}

fn prove(
    in_pipe: &mut BufReader<File>,
    out_pipe: &mut File,
    service_bin: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    // STEP 1: SPJ sends you the pipe filepath that handles the input output communication
    // STEP 2: Output your prover name, proof system name, and algorithm name
    // Note the order here: send the prover name, algorithm name, and proof system name
    write_string(out_pipe, "pcbin-plonky3-keccak")?;
    write_string(out_pipe, "STARK")?;
    write_string(out_pipe, "Plonky3")?;

    // STEP 3: Prover make all precomputes in this step
    let mut service_handler = start_service(service_bin);
    // STEP 4: Output the Number of Keccak Instances
    write_u64(out_pipe, KECCAK_INSTANCE_NUM as u64)?;
    // STEP 5: Read Input Data
    let input_bytes = read_blob(in_pipe)?;
    // STEP 6: Hash the Data
    let output = baseline_hasher(&input_bytes);
    write_byte_array(out_pipe, &output)?;
    // STEP 7: Output a String to Indicate Witness Generation Finished
    write_string(out_pipe, WITNESS_GENERATED_MSG)?;
    // STEP 8: Output the Proof
    let proof = service_handler.prove(&input_bytes);
    write_byte_array(out_pipe, &proof)?;
    let vk = vec![];
    write_byte_array(out_pipe, &vk)?;
    let pis = vec![];
    write_byte_array(out_pipe, &pis)?;

    out_pipe.flush()?;
    service_handler.stop();
    Ok(())
}

fn verify(
    in_pipe: &mut BufReader<File>,
    out_pipe: &mut File,
    service_bin: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    // STEP 9: SPJ starts your verifier by providing the pipe filepath that handles the input output communication
    let mut service_handler = start_service(service_bin);
    // STEP 10: SPJ sends the proof, verification key, and public input to the verifier
    let proof = read_blob(in_pipe)?;
    let _vk = read_blob(in_pipe)?;
    let pis = read_blob(in_pipe)?;
    // STEP 11: Verify the Proof, and send back result
    let mut result = false;
    for _ in 0..VERIFIER_REPEAT_NUM {
        result = service_handler.verify(&pis, &proof);
    }
    write_byte_array(out_pipe, &[if result { 0x00u8 } else { 0xffu8 }])?;
    write_byte_array(out_pipe, VERIFIER_REPEAT_NUM.to_le_bytes().as_ref())?; // why not number this time?

    out_pipe.flush()?;
    service_handler.stop();
    Ok(())
}

fn main() -> std::io::Result<()> {
    // parse arg
    let args = std::env::args().collect::<Vec<String>>();
    // assert_eq!(args.len(), 2, "Usage: proof-arena-integration <bin_loc> <mode:prove/verify> -toMe <in_pipe> -toSPJ <out_pipe>");
    let service_bin = &args[1];
    let mode = &args[2];
    let in_pipe_name = &args[4];
    let mut in_pipe = std::io::BufReader::new(File::open(in_pipe_name)?);
    let out_pipe_name = &args[6];
    let mut out_pipe = File::create(out_pipe_name)?;

    match mode.as_str() {
        "prove" => prove(&mut in_pipe, &mut out_pipe, service_bin).unwrap(),
        "verify" => verify(&mut in_pipe, &mut out_pipe, service_bin).unwrap(),
        _ => panic!("Invalid mode: {}", mode),
    }
    Ok(())
}

// Helper functions for SPJ communication, copied from https://github.com/sixbigsquare/proof-arena-pcbin/blob/1693b9c5d934d2364ebc259f5e413a7609cc4c27/problems/keccak256_hash/halo2/src/prover.rs

/// Writes a string to the given writer
fn write_string<W: Write>(writer: &mut W, s: &str) -> std::io::Result<()> {
    let len = s.len() as u64;
    writer.write_all(&len.to_le_bytes())?;
    writer.write_all(s.as_bytes())?;
    writer.flush()?;
    Ok(())
}

/// Writes a u64 to the given writer
fn write_u64<W: Write>(writer: &mut W, n: u64) -> std::io::Result<()> {
    writer.write_all(&n.to_le_bytes())?;
    writer.flush()?;
    Ok(())
}

/// Writes a byte array to the given writer
fn write_byte_array<W: Write>(writer: &mut W, arr: &[u8]) -> std::io::Result<()> {
    let len = arr.len() as u64;
    writer.write_all(&len.to_le_bytes())?;
    writer.write_all(arr)?;
    writer.flush()?;
    Ok(())
}

/// Reads a blob of data from the given reader
fn read_blob<R: Read>(reader: &mut R) -> std::io::Result<Vec<u8>> {
    let mut len_buf = [0u8; 8];
    reader.read_exact(&mut len_buf)?;
    let len = u64::from_le_bytes(len_buf);

    let mut buf = vec![0u8; len as usize];
    reader.read_exact(&mut buf)?;
    Ok(buf)
}
