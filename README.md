# kzg over BLS12-381
Rust library for the KZG polynomial commitment scheme, originally proposed in [this paper](http://cacr.uwaterloo.ca/techreports/2010/cacr2010-10.pdf). This implementation uses a type 2 pairing, in particular the elliptic curve [BLS12-381](https://hackmd.io/@benjaminion/bls12-381).

**WARNING**: this library has not been audited, and may not be secure against side-channel attacks. Furthermore, the only way to generate the structured reference string is through a test function. This is not secure. Do not use this library in production! 

## Some notes
- My reference for a type 2 implementation of KZG is [this blog post](https://ethresear.ch/t/yet-another-curve-but-the-curve-for-your-kzg/12861), due to Youssef El Housni. This is an excellent resource for understanding elliptic curves for KZG. Another reference that I used was Justin Thaler's [Proofs, Arguments, and Zero Knowledge](https://people.cs.georgetown.edu/jthaler/ProofsArgsAndZK.html). I took notes of select chapters from the manuscript. You can find them [here](https://github.com/alex-lindenbaum/Zero-knowledge-notes).

- This library uses [fullcodec_bls12_381](https://docs.rs/fullcodec-bls12_381/latest/fullcodec_bls12_381/index.html), which implements basic operations for BLS12-381 *and* multiscalar multiplication for $G_1$.

- The library can be split into the KZGProver and KZGVerifier. KZGProver let's you commit to a polynomial, and then commit to an evaluation of that polynomial at some arbitrary scalar. KZGVerifier can open/verify these commitments. Both structs rely heavily on the BLS12-381 library implementation.

## In the Future
This version is not final, and many additions will be made. Here is a list of some: (in no order):

- Add implementation for $N$-person trusted setup to generate the structured reference string. Currently, this library can only generate test SRS's, which should never be used in practice.

- Extending KZG for multivariate polynomials

- Add an option for an "extractable" KZG scheme, which captures the notion that if the prover $P$ succeeds in committing and revealing, then $P$ must "know" their polynomial, and isn't simply generating valid proofs. This option requires roughly twice the number of group operations, but may be useful in the case that what is required is proving *knowledge* of the secret polynomial.