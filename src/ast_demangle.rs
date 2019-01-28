use ast::*;
use std::fmt::Write;

pub trait AstDemangle {
    fn demangle_to_string(&self, out: &mut String, verbose: bool);

    fn demangle(&self, verbose: bool) -> String {
        let mut out = String::new();
        self.demangle_to_string(&mut out, verbose);
        out
    }
}

impl AstDemangle for Ident {
    fn demangle_to_string(&self, out: &mut String, verbose: bool) {
        let emit_disambiguator = match self.tag {
            IdentTag::TypeNs => {
                out.push_str(&self.ident);
                self.dis.0 != 0 && verbose
            }
            IdentTag::ValueNs => {
                out.push_str(&self.ident);

                if verbose {
                    out.push_str("'");
                }
                self.dis.0 != 0 && verbose
            }
            IdentTag::Closure => {
                out.push_str("{closure}");
                true
            }
        };

        if emit_disambiguator {
            write!(out, "[{}]", self.dis.0 + 1).unwrap();
        }
    }
}

// This should not be needed generally
impl AstDemangle for Subst {
    fn demangle_to_string(&self, out: &mut String, _verbose: bool) {
        write!(out, "{{{}}}", self.0).unwrap();
    }
}

impl AstDemangle for PathPrefix {
    fn demangle_to_string(&self, out: &mut String, verbose: bool) {
        match *self {
            PathPrefix::CrateId { ref name, ref dis } => {
                out.push_str(name);
                if verbose {
                    write!(out, "[{}]", dis).unwrap();
                }
            }
            PathPrefix::AbsolutePath {
                ref path
            } => {
                path.demangle_to_string(out, verbose);
            }
            PathPrefix::TraitImpl {
                ref self_type,
                ref impled_trait,
                dis,
            } => {
                out.push('<');
                self_type.demangle_to_string(out, verbose);
                if let &Some(ref impled_trait) = impled_trait {
                    out.push_str(" as ");
                    impled_trait.demangle_to_string(out, verbose);
                }
                out.push('>');

                if dis.0 != 0 && verbose {
                    write!(out, "[{}]", dis.0 + 1).unwrap();
                }
            }
            PathPrefix::Node {
                ref prefix,
                ref ident,
            } => {
                prefix.demangle_to_string(out, verbose);
                out.push_str("::");
                ident.demangle_to_string(out, verbose);
            }
            PathPrefix::Subst(subst) => {
                subst.demangle_to_string(out, verbose);
            }
        }
    }
}

impl AstDemangle for AbsolutePath {
    fn demangle_to_string(&self, out: &mut String, verbose: bool) {
        match *self {
            AbsolutePath::Path { ref name, ref args } => {
                name.demangle_to_string(out, verbose);
                args.demangle_to_string(out, verbose);
            }
            AbsolutePath::Subst(subst) => {
                subst.demangle_to_string(out, verbose);
            }
        }
    }
}

impl AstDemangle for GenericArgumentList {
    fn demangle_to_string(&self, out: &mut String, verbose: bool) {
        if self.len() > 0 {
            out.push('<');
            for param in self.iter() {
                param.demangle_to_string(out, verbose);
                out.push(',');
            }
            out.pop();
            out.push('>');
        }
    }
}

impl AstDemangle for Type {
    fn demangle_to_string(&self, out: &mut String, verbose: bool) {
        match *self {
            Type::BasicType(t) => {
                t.demangle_to_string(out, verbose);
            }
            Type::Ref(ref t) => {
                out.push('&');
                t.demangle_to_string(out, verbose);
            }
            Type::RefMut(ref t) => {
                out.push_str("&mut ");
                t.demangle_to_string(out, verbose);
            }
            Type::RawPtrConst(ref t) => {
                out.push_str("*const ");
                t.demangle_to_string(out, verbose);
            }
            Type::RawPtrMut(ref t) => {
                out.push_str("*mut ");
                t.demangle_to_string(out, verbose);
            }
            Type::Array(opt_size, ref t) => {
                out.push('[');
                t.demangle_to_string(out, verbose);

                if let Some(size) = opt_size {
                    write!(out, "; {}", size).unwrap();
                }

                out.push(']');
            }
            Type::Tuple(ref components) => {
                out.push('(');
                for c in components {
                    c.demangle_to_string(out, verbose);
                    out.push(',');
                }
                out.pop();
                out.push(')');
            }
            Type::Named(ref abs_path) => {
                abs_path.demangle_to_string(out, verbose);
            }
            Type::GenericParam(ref ident) => {
                ident.demangle_to_string(out, verbose);
            }
            Type::Fn {
                ref return_type,
                ref params,
                is_unsafe,
                abi,
            } => {
                if is_unsafe {
                    out.push_str("unsafe ");
                }

                if abi != Abi::Rust {
                    out.push_str("extern ");
                    abi.demangle_to_string(out, verbose);
                    out.push(' ');
                }

                out.push_str("fn(");

                if params.len() > 0 {
                    for param in params {
                        param.demangle_to_string(out, verbose);
                        out.push(',');
                    }

                    out.pop();
                }

                out.push(')');

                if let &Some(ref return_type) = return_type {
                    out.push_str(" -> ");
                    return_type.demangle_to_string(out, verbose);
                }
            }
            Type::Subst(subst) => {
                subst.demangle_to_string(out, verbose);
            }
        }
    }
}

impl AstDemangle for Abi {
    fn demangle_to_string(&self, out: &mut String, _verbose: bool) {
        out.push('"');
        match *self {
            Abi::Rust => {}
            Abi::C => out.push_str("C"),
        };
        out.push('"');
    }
}

impl AstDemangle for BasicType {
    fn demangle_to_string(&self, out: &mut String, _verbose: bool) {
        out.push_str(match *self {
            BasicType::Bool => "bool",
            BasicType::Char => "char",
            BasicType::Str => "str",
            BasicType::Unit => "()",
            BasicType::I8 => "i8",
            BasicType::I16 => "i16",
            BasicType::I32 => "i32",
            BasicType::I64 => "i64",
            BasicType::I128 => "i128",
            BasicType::Isize => "isize",
            BasicType::U8 => "u8",
            BasicType::U16 => "u16",
            BasicType::U32 => "u32",
            BasicType::U64 => "u64",
            BasicType::U128 => "u128",
            BasicType::Usize => "usize",
            BasicType::F32 => "f32",
            BasicType::F64 => "f64",
            BasicType::Never => "!",
            BasicType::Ellipsis => "...",
        });
    }
}

impl AstDemangle for Symbol {
    fn demangle_to_string(&self, out: &mut String, verbose: bool) {
        self.name.demangle_to_string(out, verbose);

        if verbose {
            if let Some(ref instantiating_crate) = self.instantiating_crate {
                out.push_str(" @ ");
                instantiating_crate.demangle_to_string(out, verbose);
            }
        }
    }
}
