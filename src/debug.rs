//! This module contains some facilities for comparing compression dictionaries.

use ast::*;
use std::cmp;

pub struct DebugDictionary {
    data: Vec<(Subst, String)>,
}

impl DebugDictionary {
    pub fn new(mut entries: Vec<(Subst, String)>) -> DebugDictionary {
        entries.sort_by_key(|&(k, _)| k);

        DebugDictionary { data: entries }
    }

    pub fn print_comparison(&self, other: &DebugDictionary) {
        let d1 = &self.data[..];
        let d2 = &other.data[..];

        if d1.len() != d2.len() {
            println!(
                "dictionaries differ in length ({} vs {})",
                d1.len(),
                d2.len()
            );
        } else {
            println!("both dictionaries have {} entries", d1.len());
        }

        for i in 0..cmp::min(d1.len(), d2.len()) {
            if d1[i] != d2[i] {
                println!("Element {} differs:", i);
                println!(" (1) {:?} => {}", d1[i].0, d1[i].1);
                println!(" (2) {:?} => {}", d2[i].0, d2[i].1);

                println!("dict 1:");
                for (k, v) in d1 {
                    let mut subst = String::new();
                    k.mangle(&mut subst);
                    println!("{} => {}", subst, v);
                }

                println!("dict 2:");
                for (k, v) in d2 {
                    let mut subst = String::new();
                    k.mangle(&mut subst);
                    println!("{} => {}", subst, v);
                }

                return;
            }
        }

        println!("dictionaries equal!");
    }
}
