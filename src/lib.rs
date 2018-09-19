
#[cfg(test)]
#[macro_use]
extern crate quickcheck;
#[cfg(test)]
extern crate rand;
#[cfg(test)]
extern crate unicode_xid;

pub mod ast;
pub mod write;
pub mod compress;

#[cfg(test)]
mod qc;
