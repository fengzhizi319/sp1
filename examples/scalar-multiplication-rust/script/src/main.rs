use std::time::Instant;
use serde::{Deserialize, Serialize};
use sp1_sdk::{include_elf, utils, ProverClient, SP1ProofWithPublicValues, SP1Stdin};

/// The ELF we want to execute inside the zkVM.
const ELF: &[u8] = include_elf!("scalar-multiplication-rust-program");


fn main() {
    // Setup a tracer for logging.
    utils::setup_logger();

    // Create an input stream.
    let mut stdin = SP1Stdin::new();

    // Generate the proof for the given program.
    let client = ProverClient::new();

    // Measure execution time of the program execution.
    let start = Instant::now();
    let (_, report) = client.execute(ELF, stdin.clone()).run().unwrap();
    let duration = start.elapsed();
    println!("executed program with {} cycles in {:?}", report.total_instruction_count(), duration);

    // Measure execution time of proof generation.
    let start = Instant::now();
    let (pk, vk) = client.setup(ELF);
    let duration = start.elapsed();
    println!("client.setup time {:?}", duration);

    let start = Instant::now();
    let mut proof = client.prove(&pk, stdin).run().unwrap();
    let duration = start.elapsed();
    println!("generated proof in {:?}", duration);


    // Read the output.
    let r = proof.public_values.read::<i32>();
    println!("r: {:?}", r);

    // Verify proof.
    let start = Instant::now();
    client.verify(&proof, &vk).expect("verification failed");
    let duration = start.elapsed();
    println!("client.verify time is {:?}", duration);

    // Test a round trip of proof serialization and deserialization.
    proof.save("proof-with-pis.bin").expect("saving proof failed");
    let deserialized_proof =
        SP1ProofWithPublicValues::load("proof-with-pis.bin").expect("loading proof failed");

    // Verify the deserialized proof.
    let start = Instant::now();
    client.verify(&deserialized_proof, &vk).expect("verification failed");
    let duration = start.elapsed();
    println!("deserialized proof verify time is {:?}", duration);

    println!("successfully generated and verified proof for the program!")
}
