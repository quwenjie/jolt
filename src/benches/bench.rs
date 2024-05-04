use crate::lasso::surge::SparsePolyCommitmentGens;
use crate::subtables::and::AndSubtableStrategy;
use crate::{
  lasso::{densified::DensifiedRepresentation, surge::SparsePolynomialEvaluationProof},
  utils::random::RandomTape,
};
use ark_bls12_381::{G1Projective, Fr};
use ark_std::One;
use ark_ff::PrimeField;
use ark_std::{log2, test_rng};
use merlin::Transcript;
use rand_chacha::rand_core::RngCore;
use std::str::FromStr;
use crate::subtables::range_check::RangeCheckSubtableStrategy;



pub fn gen_random_points<F: PrimeField, const C: usize>(memory_bits: usize) -> [Vec<F>; C] {
  std::array::from_fn(|_| gen_random_point(memory_bits))
}

pub fn gen_random_point<F: PrimeField>(memory_bits: usize) -> Vec<F> {
  let mut rng = test_rng();
  let mut r_i: Vec<F> = Vec::with_capacity(memory_bits);
  for _ in 0..memory_bits {
    r_i.push(F::rand(&mut rng));
  }
  r_i
}
pub fn gen_indices_ours<const C: usize>(sparsity: usize, memory_size: usize) -> Vec<[usize; C]> {
  let mut rng = test_rng();
  let mut all_indices: Vec<[usize; C]> = Vec::new();
  for i in 0..sparsity 
  {
    let mut indices = [0 as usize % memory_size; C];
    indices[0]=i+1;
    if(i==0)
    {
      indices[1]=1;
    }
    all_indices.push(indices);
  }
  all_indices
}

macro_rules! single_pass_lasso {
  ($span_name:expr, $field:ty, $group:ty, $subtable_strategy:ty, $C:expr, $M:expr, $sparsity:expr) => {
    (tracing::info_span!($span_name), move || {
      const C: usize = $C;
      const M: usize = $M;
      const S: usize = $sparsity;
      type F = $field;
      type G = $group;
      type SubtableStrategy = $subtable_strategy;

      let log_m = log2(M) as usize;
      let log_s: usize = log2($sparsity) as usize;

      let mut r: Vec<F> = gen_random_point::<F>(log_s);
      
      for i in 0..log_s
      {
        if(i<3)
        {  
          r[i]=F::from_str("150").unwrap();
        }
        else
        {
          r[i]=F::from_str("158").unwrap();
        }
        println!("{}",r[i]);
      }
      
      let a:F=F::from_str("52435875175126190479447740508185965837690552500527637822603658699938581184504").unwrap() ;
      let b:F=F::from_str("14").unwrap();
      println!("haha :{} ",a+b);
      let nz = gen_indices_ours::<C>(S, M);

      // Prove
      let mut dense: DensifiedRepresentation<F, C> =
        DensifiedRepresentation::from_lookup_indices(&nz, log_m);
      let gens = SparsePolyCommitmentGens::<G>::new(b"gens_sparse_poly", C, S, C, log_m);
      let commitment = dense.commit::<$group>(&gens);
      let mut random_tape = RandomTape::new(b"proof");
      let mut prover_transcript = Transcript::new(b"example");
      
      let proof = SparsePolynomialEvaluationProof::<G, C, M, SubtableStrategy>::prove(
        &mut dense,
        &r,
        &gens,
        &mut prover_transcript,
        &mut random_tape,
      );
      let mut verify_transcript = Transcript::new(b"example");
      proof
        .verify(&commitment, &r, &gens, &mut verify_transcript)
        .expect("should verify");
    })
  };
}

#[derive(Debug, Clone, clap::ValueEnum)]
pub enum BenchType {
  Halo2Comparison,
}

#[allow(unreachable_patterns)] // good errors on new BenchTypes
pub fn benchmarks(bench_type: BenchType) -> Vec<(tracing::Span, fn())> {
  match bench_type {
    BenchType::Halo2Comparison => halo2_comparison_benchmarks(),
    _ => panic!("BenchType does not have a mapping"),
  }
}


fn halo2_comparison_benchmarks() -> Vec<(tracing::Span, fn())> 
{
  vec![
    single_pass_lasso!(
      "Range check ",
      Fr,
      G1Projective,
      RangeCheckSubtableStrategy::<24>,
      /* C= */ 3,
      /* M= */ 1 << 16,
      /* S= */ 1 << 6
    ),
  ]
}
