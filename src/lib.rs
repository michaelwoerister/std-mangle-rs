
#[cfg(test)]
#[macro_use]
extern crate quickcheck;
#[cfg(test)]
extern crate rand;
#[cfg(test)]
extern crate unicode_xid;

pub mod ast;
pub mod compress;
pub mod debug;
pub mod decompress;
pub mod mangle;
pub mod parse;
pub mod pretty;
pub mod write;

#[cfg(test)]
mod generated_tests;

#[cfg(test)]
mod qc;
