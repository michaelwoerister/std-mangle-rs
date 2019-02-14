extern crate unic_idna_punycode as punycode;

#[cfg(test)]
// #[macro_use]
extern crate quickcheck;
#[cfg(test)]
extern crate rand;

pub mod ast;

// pub mod compress;
// pub mod compress_alt;
// pub mod decompress;

pub mod ast_demangle;
// pub mod direct_demangle;
// pub mod mangle;
pub mod parse;

mod charset;
mod error;
pub mod int_radix;
// mod same;

// #[cfg(test)]
// mod debug;
#[cfg(test)]
mod generated_tests;

// #[cfg(test)]
// mod quickcheck_testing;

// /// Demangles a mangled symbol name, doing parsing, decompression, and
// /// pretty-printing in one pass. This is the most efficient way of demangling
// /// a symbol. Returns `Err` if the symbol cannot be parsed or an invalid
// /// substitution index is encountered.
// pub fn demangle_symbol(mangled_symbol: &str, verbose: bool) -> Result<String, String> {
//     direct_demangle::Demangler::demangle(mangled_symbol.as_bytes(), verbose)
// }

// /// Generates the mangled version of a symbol name's AST.
// pub fn ast_to_mangled_symbol(symbol_ast: &ast::Symbol) -> String {
//     let mut output = String::new();
//     symbol_ast.mangle(&mut output);
//     output
// }

/// Construct the AST for a mangled symbol name.
pub fn mangled_symbol_to_ast(mangled_symbol: &str) -> Result<ast::Symbol, String> {
    parse::parse(mangled_symbol.as_bytes())
}

/// Generates the demangled version of a symbol name's AST.
pub fn ast_to_demangled_symbol(symbol_ast: &ast::Symbol) -> String {
    ast_demangle::AstDemangle::demangle(symbol_ast)
}

// /// Compresses a symbol name's AST by replacing already encounters sub-trees
// /// with substitutions. Panics if the AST already contains substitutions.
// pub fn compress_ast(symbol_ast: &ast::Symbol) -> ast::Symbol {
//     compress::compress_ext(symbol_ast).0
// }

// /// Decompresses a symbol name's AST by expanding substitutions to the sub-trees
// /// they refer to. Panics if the AST is not well-formed (i.e. if a substitution
// /// index is encountered that cannot be mapped to a sub-tree).
// pub fn decompress_ast(symbol_ast: &ast::Symbol) -> ast::Symbol {
//     decompress::decompress_ext(symbol_ast).0
// }
