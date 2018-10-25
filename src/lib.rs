
#[cfg(test)]
#[macro_use]
extern crate quickcheck;
#[cfg(test)]
extern crate rand;
#[cfg(test)]
extern crate unicode_xid;

pub mod ast;
pub mod compress;
pub mod mangle;
pub mod write;

#[cfg(test)]
mod qc;
