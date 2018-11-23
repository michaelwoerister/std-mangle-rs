use ast::*;
use std::fmt::Write;


pub trait AstDemangle {
    fn demangle_to_string(&self, out: &mut String);

    fn demangle(&self) -> String {
        let mut out = String::new();
        self.demangle_to_string(&mut out);
        out
    }
}

impl AstDemangle for Ident {
    fn demangle_to_string(&self, out: &mut String) {
        let emit_disambiguator = match self.tag {
            IdentTag::TypeNs => {
                out.push_str(&self.ident);
                self.dis.0 != 0
            }
            IdentTag::ValueNs => {
                out.push_str(&self.ident);
                out.push_str("'");
                self.dis.0 != 0
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
    fn demangle_to_string(&self, out: &mut String) {
        write!(out, "{{{}}}", self.0).unwrap();
    }
}

impl AstDemangle for NamePrefix {
    fn demangle_to_string(&self, out: &mut String) {
        match *self {
            NamePrefix::CrateId { ref name, ref dis } => {
                write!(out, "{}[{}]", name, dis).unwrap();
            }
            NamePrefix::TraitImpl {
                ref self_type,
                ref impled_trait,
                dis,
            } => {
                out.push('<');
                self_type.demangle_to_string(out);
                out.push_str(" as ");
                impled_trait.demangle_to_string(out);
                out.push('>');

                if dis.0 != 0 {
                    write!(out, "[{}]", dis.0 + 1).unwrap();
                }
            }
            NamePrefix::InherentImpl { ref self_type } => {
                self_type.demangle_to_string(out);
            }
            NamePrefix::Node {
                ref prefix,
                ref ident,
            } => {
                prefix.demangle_to_string(out);
                out.push_str("::");
                ident.demangle_to_string(out);
            }
            NamePrefix::Subst(subst) => {
                subst.demangle_to_string(out);
            }
        }
    }
}

impl AstDemangle for QName {
    fn demangle_to_string(&self, out: &mut String) {
        match *self {
            QName::Name { ref name, ref args } => {
                name.demangle_to_string(out);
                args.demangle_to_string(out);
            }
            QName::Subst(subst) => {
                subst.demangle_to_string(out);
            }
        }
    }
}

impl AstDemangle for GenericArgumentList {
    fn demangle_to_string(&self, out: &mut String) {
        if self.len() > 0 {
            out.push('<');
            for param in self.iter() {
                param.demangle_to_string(out);
                out.push(',');
            }
            out.pop();
            out.push('>');
        }
    }
}

impl AstDemangle for Type {
    fn demangle_to_string(&self, out: &mut String) {
        match *self {
            Type::BasicType(t) => {
                t.demangle_to_string(out);
            }
            Type::Ref(ref t) => {
                out.push('&');
                t.demangle_to_string(out);
            }
            Type::RefMut(ref t) => {
                out.push_str("&mut ");
                t.demangle_to_string(out);
            }
            Type::RawPtrConst(ref t) => {
                out.push_str("*const ");
                t.demangle_to_string(out);
            }
            Type::RawPtrMut(ref t) => {
                out.push_str("*mut ");
                t.demangle_to_string(out);
            }
            Type::Array(opt_size, ref t) => {
                out.push('[');
                t.demangle_to_string(out);

                if let Some(size) = opt_size {
                    write!(out, "; {}", size).unwrap();
                }

                out.push(']');
            }
            Type::Tuple(ref components) => {
                out.push('(');
                for c in components {
                    c.demangle_to_string(out);
                    out.push(',');
                }
                out.pop();
                out.push(')');
            }
            Type::Named(ref qname) => {
                qname.demangle_to_string(out);
            }
            Type::GenericParam(ref ident) => {
                ident.demangle_to_string(out);
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
                    abi.demangle_to_string(out);
                    out.push(' ');
                }

                out.push_str("fn(");

                if params.len() > 0 {
                    for param in params {
                        param.demangle_to_string(out);
                        out.push(',');
                    }

                    out.pop();
                }

                out.push(')');

                if let &Some(ref return_type) = return_type {
                    out.push_str(" -> ");
                    return_type.demangle_to_string(out);
                }
            }
            Type::Subst(subst) => {
                subst.demangle_to_string(out);
            }
        }
    }
}

impl AstDemangle for Abi {
    fn demangle_to_string(&self, out: &mut String) {
        out.push('"');
        match *self {
            Abi::Rust => {}
            Abi::C => out.push_str("C"),
        };
        out.push('"');
    }
}

impl AstDemangle for BasicType {
    fn demangle_to_string(&self, out: &mut String) {
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
    fn demangle_to_string(&self, out: &mut String) {
        self.name.demangle_to_string(out);

        if let Some(ref instantiating_crate) = self.instantiating_crate {
            out.push_str(" @ ");
            instantiating_crate.demangle_to_string(out);
        }
    }
}
