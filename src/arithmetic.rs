use fullcodec_bls12_381::{G1Projective, BlsScalar, G2Projective, G2Affine};

use crate::utils::from_i64_to_scalars;

pub type G1 = G1Projective;
pub type G2 = G2Projective;
pub type Scalar = BlsScalar;

pub fn eval_polynomial(degree: u64, coeffs: &Vec<Scalar>, x: &Scalar) -> Scalar {
    let mut sum = Scalar::zero();

    for i in 0..degree + 1 {
        sum += coeffs[i as usize] * x.pow(&[i, 0, 0, 0]);
    }

    sum
}

// The following code is a slight modification of fullcodec_bls12_381::multiscalar_mul::msm_variable_base src
// Allows us to take exponents of points in G2. This is needed for verification since we are using a type 2 pairing
// Original src: https://docs.rs/fullcodec-bls12_381/latest/src/fullcodec_bls12_381/multiscalar_mul.rs.html#1-319
pub fn msm_g2(points: &[G2Affine], scalars: &[Scalar]) -> G2Projective {
    let c = if scalars.len() < 32 {
        3
    } else {
        ln_without_floats(scalars.len()) + 2
    };

    let num_bits = 255usize;
    let fr_one = Scalar::one();

    let zero = G2Projective::identity();
    let window_starts: Vec<_> = (0..num_bits).step_by(c).collect();

    let window_starts_iter = window_starts.into_iter();

    // Each window is of size `c`.
    // We divide up the bits 0..num_bits into windows of size `c`, and
    // in parallel process each such window.
    let window_sums: Vec<_> = window_starts_iter
        .map(|w_start| {
            let mut res = zero;
            // We don't need the "zero" bucket, so we only have 2^c - 1 buckets
            let mut buckets = vec![zero; (1 << c) - 1];
            scalars
                .iter()
                .zip(points)
                .filter(|(s, _)| !(*s == &Scalar::zero()))
                .for_each(|(&scalar, base)| {
                    if scalar == fr_one {
                        // We only process unit scalars once in the first window.
                        if w_start == 0 {
                            res = res.add_mixed(base);
                        }
                    } else {
                        let mut scalar = scalar.reduce();

                        // We right-shift by w_start, thus getting rid of the
                        // lower bits.
                        scalar.divn(w_start as u32);

                        // We mod the remaining bits by the window size.
                        let scalar = scalar.0[0] % (1 << c);

                        // If the scalar is non-zero, we update the corresponding
                        // bucket.
                        // (Recall that `buckets` doesn't have a zero bucket.)
                        if scalar != 0 {
                            buckets[(scalar - 1) as usize] =
                                buckets[(scalar - 1) as usize].add_mixed(base);
                        }
                    }
                });

            let mut running_sum = G2Projective::identity();
            for b in buckets.into_iter().rev() {
                running_sum = running_sum + b;
                res += &running_sum;
            }

            res
        })
        .collect();

    // We store the sum for the lowest window.
    let lowest = *window_sums.first().unwrap();
    // We're traversing windows from high to low.
    window_sums[1..]
        .iter()
        .rev()
        .fold(zero, |mut total, sum_i| {
            total += sum_i;
            for _ in 0..c {
                total = total.double();
            }
            total
        })
        + lowest
}

fn ln_without_floats(a: usize) -> usize {
    // log2(a) * ln(2)
    (log2(a) * 69 / 100) as usize
}
fn log2(x: usize) -> u32 {
    if x <= 1 {
        return 0;
    }

    let n = x.leading_zeros();
    core::mem::size_of::<usize>() as u32 * 8 - n
}

pub fn monic_division(coeffs: &Vec<Scalar>, denom: &Scalar) -> Vec<Scalar> {
    let mut index = coeffs.len() - 1;
    let mut new_coeffs = vec![coeffs[index]];

    while index > 1 {
        new_coeffs.insert(
            0, 
            denom * new_coeffs[0] + coeffs[index - 1]);
        index -= 1;
    }

    new_coeffs
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_monic_division() {
        let coeffs = from_i64_to_scalars(&[5, 2, 3, 4]);
        let denom = Scalar::from(1);
        let new_coeffs = monic_division(&coeffs, &denom);

        println!("TESTING COEFFICIENTS");
        println!("{}", new_coeffs[0] == Scalar::from(9));
    }
}