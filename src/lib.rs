extern crate unic_idna_punycode as punycode;

#[cfg(test)]
#[macro_use]
extern crate quickcheck;
#[cfg(test)]
extern crate rand;

pub mod ast;
pub mod ast_demangle;
pub mod parse;

mod charset;
mod error;
pub mod int_radix;

#[cfg(test)]
mod generated_tests;

/// Construct the AST for a mangled symbol name.
pub fn mangled_symbol_to_ast(mangled_symbol: &str) -> Result<ast::Symbol, String> {
    parse::parse(mangled_symbol.as_bytes())
}

/// Generates the demangled version of a symbol name's AST.
pub fn ast_to_demangled_symbol(symbol_ast: &ast::Symbol) -> String {
    ast_demangle::AstDemangle::demangle(symbol_ast)
}
