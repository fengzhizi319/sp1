use std::time::Instant;
use sp1_sdk::{include_elf, utils, ProverClient, SP1ProofWithPublicValues, SP1Stdin};

/// The ELF we want to execute inside the zkVM.
const ELF: &[u8] = include_elf!("fibonacci-program");

fn main() {
    // Setup logging.
    utils::setup_logger();

    // Create an input stream and write '500' to it.
    let n = 1u32;

    // The input stream that the program will read from using `sp1_zkvm::io::read`. Note that the
    // types of the elements in the input stream must match the types being read in the program.
    let mut stdin = SP1Stdin::new();
    stdin.write(&n);

    // Create a `ProverClient` method.
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

    // Read and verify the output.
    //
    // Note that this output is read from values committed to in the program using
    // `sp1_zkvm::io::commit`.
    let _ = proof.public_values.read::<u32>();
    let a = proof.public_values.read::<u32>();
    let b = proof.public_values.read::<u32>();

    println!("a: {}", a);
    println!("b: {}", b);

    // Measure execution time of proof verification.
    let start = Instant::now();
    client.verify(&proof, &vk).expect("verification failed");
    let duration = start.elapsed();
    println!("verified proof in {:?}", duration);

    // Test a round trip of proof serialization and deserialization.
    proof.save("proof-with-pis.bin").expect("saving proof failed");
    let deserialized_proof =
        SP1ProofWithPublicValues::load("proof-with-pis.bin").expect("loading proof failed");

    // Measure execution time of deserialized proof verification.
    let start = Instant::now();
    client.verify(&deserialized_proof, &vk).expect("verification failed");
    let duration = start.elapsed();
    println!("verified deserialized proof in {:?}", duration);

    println!("successfully generated and verified proof for the program!")
}