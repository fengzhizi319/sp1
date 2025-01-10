#![no_main]
sp1_zkvm::entrypoint!(main);

use std::str::FromStr;
use banderwagonrust::{Element, Field, Fr};
use banderwagonrust::salt_committer::Committer;
use serde::{Deserialize, Serialize};
use sp1_zkvm::syscalls::sys_bigint;
pub(crate) const BIGINT_WIDTH_WORDS: usize = 8;
fn test_mon_mul() {
    println!("test Montgomery multiplication");
    let modulus: [u32; BIGINT_WIDTH_WORDS] = [0x00000001,0xffffffff, 0xfffe5bfe, 0x53bda402, 0x09a1d805, 0x3339d808, 0x299d7d48, 0x73eda753];
    let fq: [u32; BIGINT_WIDTH_WORDS] = [0xffffffff, 0x00000001, 0xfffe5bfe, 0x53bda402, 0x09a1d805, 0x3339d808, 0x299d7d48, 0x73eda753];

    let inv_r: [u32; BIGINT_WIDTH_WORDS] = [0xfe75c040, 0x13f75b69, 0x09dc705f, 0xab6fca8f, 0x4f77266a, 0x7204078a, 0x30009d57, 0x1bbe8693];
    let inv_r1: [u32; BIGINT_WIDTH_WORDS] = [0xfe75c040,0x13f75b69,0x09dc705f,0xab6fca8f,0x4f77266a,0x7204078a,0x30009d57,0x1bbe8693];
    let r: [u32; BIGINT_WIDTH_WORDS] = [0xfffffffe, 0x00000001, 0x00034802, 0x5884b7fa, 0xecbc4ff5, 0x998c4fef, 0xacc5056f, 0x1824b159];
    let test_a: [u32; BIGINT_WIDTH_WORDS] = [0x3AE735D7, 0x4A2B0F46, 0xB55D95F3, 0x1C1A1D69, 0x57C59A5B, 0x708B232A, 0x1B58C712, 0x3012C00A];

    let mut bigint_1: [u32; BIGINT_WIDTH_WORDS] = [1, 0u32, 0u32, 0u32, 0u32, 0u32, 0u32, 0u32];
    let mut bigint_b: [u32; BIGINT_WIDTH_WORDS] = [0, 0x00000002, 0xfffe5bfe, 0x53bda402, 0x09a1d805, 0x3339d808, 0x299d7d48, 0x73eda753];
    let mut result1: [u32; BIGINT_WIDTH_WORDS] = [0u32; BIGINT_WIDTH_WORDS];
    let mut result2: [u32; BIGINT_WIDTH_WORDS] = [0u32; BIGINT_WIDTH_WORDS];
    let mut result3: [u32; BIGINT_WIDTH_WORDS] = [0u32; BIGINT_WIDTH_WORDS];
    let mut result4: [u32; BIGINT_WIDTH_WORDS] = [0u32; BIGINT_WIDTH_WORDS];

    let one: [u32; BIGINT_WIDTH_WORDS] = [0xfffffffe, 0x00000001, 0x00034802, 0x5884b7fa, 0xecbc4ff5, 0x998c4fef, 0xacc5056f, 0x1824b159];
    let a: [u32; BIGINT_WIDTH_WORDS] = [172325, 12, 12, 12, 12, 12, 12, 34];
    let mut z: [u32; BIGINT_WIDTH_WORDS] = [0; BIGINT_WIDTH_WORDS];
    let mut out: [u32; BIGINT_WIDTH_WORDS] = [0; BIGINT_WIDTH_WORDS];
    unsafe {
        sys_bigint(
            &mut z,
            0,
            &one,
            &a,
            &modulus,
        );
        sys_bigint(
            &mut out,
            0,
            &z,
            &inv_r,
            &modulus,
        );
    }
    println!("one*a*invR is {:X?}", out);
    unsafe {
        sys_bigint(
            &mut result4,
            0,
            &bigint_1,
            &bigint_b,
            &modulus,
        );
    }
    println!("1*1 is {:X?}", result4);

    unsafe {
        sys_bigint(
            &mut result3,
            0,
            &inv_r1,
            &r,
            &modulus,
        );
    }
    println!("R*invR is {:X?}", result3);
    unsafe {
        sys_bigint(
            &mut result1,
            0,
            &bigint_1,
            &r,
            &modulus,
        );
    }
    println!("aR is {:X?}", result1);
    //[FFFFFFFE, 1, 34802, 5884B7FA, ECBC4FF5, 998C4FEF, ACC5056F, 1824B159]
    unsafe {
        sys_bigint(
            &mut result2,
            0,
            &result1,
            &inv_r,
            &modulus,
        );
    }
    println!("aR*invR is {:X?}", result2);
    //[EA074064, B385237B, B9B4EB32, 9794A28D, B2BA30C, C5E88A00, CBDC233F, 1C535D75]
}
fn correctness_banderwagon_for_debug() {

    // Create a vector of 256 elements, each being a multiple of the prime subgroup generator

    let basis_num = 1;
    let mut basic_crs = Vec::with_capacity(basis_num);
    for i in 0..basis_num {
        basic_crs.push(Element::prime_subgroup_generator() * Fr::from((i + 1) as u64));
    }
    // log(&format!("basic_crs: {:?}", basic_crs));
    //q-1
    let scalar = Fr::from_str(
        "13108968793781547619861935127046491459309155893440570251786403306729687672800",
    )
        .unwrap();
    // log(&format!("scalar: {:?}", scalar));

    let precompute = Committer::new(&basic_crs, 5);
    // log(&format!("precompute"));

    let got_result = precompute.mul_index(&scalar, 0);
    // log(&format!("got_result: {:?}", got_result));
    let rx=got_result.0.x;
    let ry=got_result.0.y;
    let rz=got_result.0.z;
    let z_inv = rz.inverse().unwrap();
    let x = rx * &z_inv;
    let y = ry * &z_inv;
}
pub fn main() {
    let a=3;
    let b=4;
    let out=a+b;
    println!("Addition of 3+4 =: {:?}", out);
    test_mon_mul();
    correctness_banderwagon_for_debug();
    sp1_zkvm::io::commit(&out);
}
