use fullcodec_bls12_381::{BlsScalar, G1Affine, G2Affine};

use crate::arithmetic::{G1, G2};

pub fn from_i64_to_scalars(coeffs: &[i64]) -> Vec<BlsScalar> {
    coeffs.iter()
        .map(|&c| {
            if c < 0 {
                let c_neg = BlsScalar::from(-c as u64);
                return c_neg.neg()
            }

            return BlsScalar::from(c as u64);
        })
        .collect()
}

pub fn _print_g1(ctx: &str, g: &G1) {
    let g_affine: G1Affine = g.into();
    let bytes = g_affine.to_raw_bytes();
    let hex = hex::encode(bytes);

    println!("{}: {}", ctx, hex);
}

pub fn _print_g2(ctx: &str, g: &G2) {
    let g_affine: G2Affine = g.into();
    let bytes = g_affine.to_raw_bytes();
    let hex = hex::encode(bytes);

    println!("{}: {}", ctx, hex);
}