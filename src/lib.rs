use fullcodec_bls12_381::{multiscalar_mul::msm_variable_base, pairing, G1Affine};

use crate::arithmetic::{monic_division, G1, G2, Scalar, eval_polynomial, msm_g2};

#[allow(unused_imports)]
mod arithmetic;
mod utils;

pub struct PointCommitment {
    pub u: Scalar,
    pub v: Scalar,
    pub pi: G1
}

pub struct KZGProver {
    degree: u64,
    srs: Vec<G1Affine>,
    coeffs: Vec<Scalar>
}

impl KZGProver {
    pub fn generate_poly_commitment(&self) -> G1 {
        // g^q(tau) = \sum_i (g^tau^i)^q_i
        msm_variable_base(
            &self.srs,
            &self.coeffs
        )
    }

    pub fn generate_point_commitment(&self, u: &Scalar) -> PointCommitment {
        let v = eval_polynomial(self.degree, &self.coeffs, u);

        // Compute witness polynomial w(X) = (q(X) - v) / (X - u), using synthetic division over q(X) - v
        // In fact, we don't need to compute q(X), since we know that (X - u) divides q(X) - v. Remainder will be 0 so we ignore the constant term
        let witness_coeffs = monic_division(&self.coeffs, u);

        // Compute g^w(tau)
        let pi = msm_variable_base(
            &self.srs,
            &witness_coeffs
        );

        PointCommitment { u: *u, v, pi }
    }
}

pub struct KZGVerifier {
    vk: Vec<G2>, // verifier only needs g and g^tau (elements of G_2)
    g1_generator: G1Affine,
    poly_commitment: G1,
    point_commitment: PointCommitment,
}

impl KZGVerifier {
    pub fn verify_point_commitment(&self) -> bool {
        let u_inv = self.point_commitment.u.neg();
        let v_inv = self.point_commitment.v.neg();

        let g2_u = msm_g2(&[self.vk[0].into()], &[u_inv]);
        let g1_v = msm_variable_base(&[self.g1_generator.into()], &[v_inv]);

        let lhs = pairing(&self.point_commitment.pi.into(), &(self.vk[1] + g2_u).into());
        let rhs = pairing(&(self.poly_commitment + g1_v).into(), &self.vk[0].into());

        lhs == rhs
    }
}

pub fn generate_secrets_test(degree: u64) -> (Vec<G1Affine>, Vec<G2>) {
    let g = fullcodec_bls12_381::G1Affine::generator();     // msm_variable_base takes G1Affine representations

    let rng = rand::thread_rng();
    let tau = Scalar::random(rng);      // Probability that tau = 0 is ~1/2^381

    let mut srs: Vec<G1Affine> = Vec::new();
    let mut exp = Scalar::one();
    for _i in 0..=degree {
        srs.push(msm_variable_base(&[g], &[exp]).into());
        exp *= tau;
    }

    let g2 = G2::generator();
    let g2_tau = msm_g2(&[g2.into()], &[tau]);
    let vk = vec![g2, g2_tau];

    (srs, vk)
}

#[cfg(test)]
mod test {
    use colored::Colorize;
    use fullcodec_bls12_381::G1Affine;

    use super::*;

    #[test]
    fn commit_and_reveal() {
        let num_srs_changes = 1;
        let num_polys_per_srs = 5;
        let d = 20;

        for i in 1..=num_srs_changes {
            let (srs, vk) = generate_secrets_test(d);
            
            for j in 1..=num_polys_per_srs {
                print!("SRS #{i}. Setting up polynomial #{j}. ");

                let mut rng = rand::thread_rng();
                
                // Generate d + 2 scalars: d + 1 for polynomial and 1 for input u
                let u = Scalar::random(rng);
                let mut coeffs: Vec<Scalar> = Vec::new();
                for _k in 0..=d {
                    rng = rand::thread_rng(); // TODO: find a way to move rng out of this scope
                    coeffs.push(Scalar::random(rng));
                }

                let prover = KZGProver {
                    degree: d,
                    srs: srs.clone(),
                    coeffs
                };

                let poly_commitment = prover.generate_poly_commitment();
                let point_commitment = prover.generate_point_commitment(&u);

                let verifier = KZGVerifier {
                    vk: vk.clone(),
                    g1_generator: srs[0],
                    poly_commitment,
                    point_commitment
                };

                let out = verifier.verify_point_commitment();

                if out {
                    println!("{}", "q(u) = v commitment accepted.".green());
                } else {
                    println!("{}", "q(u) = v commitment rejected.".red());
                }

                assert!(out);
            }
        }
    }

    #[test]
    fn false_point_commitment() {
        let num_srs_changes = 10;
        let num_polys_per_srs = 5;
        let d = 20;

        for i in 1..=num_srs_changes {
            let (srs, vk) = generate_secrets_test(d);
            
            for j in 1..=num_polys_per_srs {
                print!("SRS #{i}. Setting up polynomial #{j}. ");

                let mut rng = rand::thread_rng();
                
                // Generate d + 2 scalars: d + 1 for polynomial and 1 for input u
                let u = Scalar::random(rng);
                let mut coeffs: Vec<Scalar> = Vec::new();
                for _k in 0..=d {
                    rng = rand::thread_rng(); // TODO: find a way to move rng out of this scope
                    coeffs.push(Scalar::random(rng));
                }

                let prover = KZGProver {
                    degree: d,
                    srs: srs.clone(),
                    coeffs
                };

                let poly_commitment = prover.generate_poly_commitment();
                
                // We pick v_false at random, accepting that with small prob. v_false = q(u)
                rng = rand::thread_rng();
                let v_false = Scalar::random(rng);

                // A 'fake proof' cannot be naturally generated, since the witness would not be a polynomial 
                // So we test the verifier on a random G1 element for pi
                rng = rand::thread_rng();
                let exp = Scalar::random(rng);
                let pi_false = msm_variable_base(&[G1Affine::generator()], &[exp]);
                let point_commitment = PointCommitment { u, v: v_false, pi: pi_false };

                let verifier = KZGVerifier {
                    vk: vk.clone(),
                    g1_generator: srs[0],
                    poly_commitment,
                    point_commitment
                };

                let out = !verifier.verify_point_commitment();

                if out {
                    println!("{}", "Verifier rejected false proof.".green());
                } else {
                    println!("{}", "Verifier accepted false proof.".red());
                }

                assert!(out);
            }
        }
    }
}