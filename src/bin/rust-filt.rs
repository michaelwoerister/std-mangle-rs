
extern crate std_mangle_rs;

use std_mangle_rs::{mangled_symbol_to_ast, ast_to_demangled_symbol};

fn main() {
    let args: Vec<_> = std::env::args().collect();

    if args.len() >= 2 {

        let ast = mangled_symbol_to_ast(&args[1]).unwrap();
        let demangled = ast_to_demangled_symbol(&ast);
        println!("{}", demangled);
    } else {
        eprintln!("no arguments found");
    }
}