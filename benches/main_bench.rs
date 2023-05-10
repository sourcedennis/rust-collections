// stdlib imports
use std::{collections::{BinaryHeap}, cmp::Reverse};
// external library imports
use rand::prelude::*;
use criterion::{criterion_group, criterion_main, Criterion};
use min_max_heap::MinMaxHeap;


/// Add `n` numbers, and pop them in increasing order.
#[inline]
fn criterion_benchmark_min(c: &mut Criterion, n: usize) {
  let mut nums: Vec< u32 > = Vec::with_capacity( n );
  let mut rng = rand::thread_rng( );
  for _ in 0..n {
    nums.push( rng.gen( ) );
  }

  let mut g = c.benchmark_group( format!( "min{}", n ) );

  g.bench_function( "stdlib", |b|
    b.iter( || {
      let mut h: BinaryHeap< Reverse< u32 > > = BinaryHeap::default( );
      for v in &nums {
        h.push( Reverse( *v ) );
      }
      for _ in 0..n {
        h.pop( );
      }
    } )
  );

  g.bench_function( "min_max_heap", |b|
    b.iter( || {
      let mut h: MinMaxHeap< u32 > = MinMaxHeap::default( );
      for v in &nums {
        h.push( *v );
      }
      for _ in 0..n {
        h.pop_min( );
      }
    } )
  );

  g.bench_function( "local", |b|
    b.iter( || {
      let mut h: rust_collections::MinMaxHeap< u32 > = rust_collections::MinMaxHeap::default( );
      for v in &nums {
        h.push( *v );
      }
      for _ in 0..n {
        h.pop_min( );
      }
    } )
  );
}

/// Add `n` numbers, and pop them in decreasing order.
#[inline]
fn criterion_benchmark_max(c: &mut Criterion, n: usize) {
  let mut nums: Vec< u32 > = Vec::with_capacity( n );
  let mut rng = rand::thread_rng( );
  for _ in 0..n {
    nums.push( rng.gen( ) );
  }

  let mut g = c.benchmark_group( format!( "max{}", n ) );

  g.bench_function( "stdlib", |b|
    b.iter( || {
      let mut h: BinaryHeap< u32 > = BinaryHeap::default( );
      for v in &nums {
        h.push( *v );
      }
      for _ in 0..n {
        h.pop( );
      }
    } )
  );

  g.bench_function( "min_max_heap", |b|
    b.iter( || {
      let mut h: MinMaxHeap< u32 > = MinMaxHeap::default( );
      for v in &nums {
        h.push( *v );
      }
      for _ in 0..n {
        h.pop_max( );
      }
    } )
  );

  g.bench_function( "local", |b|
    b.iter( || {
      let mut h: rust_collections::MinMaxHeap< u32 > = rust_collections::MinMaxHeap::default( );
      for v in &nums {
        h.push( *v );
      }
      for _ in 0..n {
        h.pop_max( );
      }
    } )
  );
}

fn criterion_benchmark100( c: &mut Criterion ) {
  criterion_benchmark_min( c, 100 );
  criterion_benchmark_max( c, 100 );
}

fn criterion_benchmark10_000( c: &mut Criterion ) {
  criterion_benchmark_min( c, 10_000 );
  criterion_benchmark_max( c, 10_000 );
}

fn criterion_benchmark_min1_000_000( c: &mut Criterion ) {
  criterion_benchmark_min( c, 1_000_000 );
  criterion_benchmark_max( c, 1_000_000 );
}

criterion_group!(benches, criterion_benchmark100, criterion_benchmark10_000, criterion_benchmark_min1_000_000);
criterion_main!(benches);
