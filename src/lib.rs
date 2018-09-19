#[cfg(test)]
#[macro_use]
extern crate quickcheck;
#[cfg(test)]
extern crate rand;
#[cfg(test)]
extern crate unicode_xid;

pub mod ast;

pub mod compress;
pub mod decompress;

pub mod direct_demangle;
pub mod mangle;
pub mod parse;
pub mod pretty;

mod error;
mod same;

#[cfg(test)]
mod debug;
#[cfg(test)]
mod generated_tests;
#[cfg(test)]
mod qc;
