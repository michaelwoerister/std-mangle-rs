

use ast::*;
use std::fmt::Write;

impl IdentTag {
    pub fn pretty_print(&self, out: &mut String) {
        match *self {
            IdentTag::TypeNs => {},
            IdentTag::ValueNs => {},
            IdentTag::Closure => {},
        };
    }
}

impl Ident {
    pub fn pretty_print(&self, out: &mut String) {
        out.push_str(&self.ident);

        self.tag.pretty_print(out);

        if self.dis != 0 {
            write!(out, "'{}", self.dis + 1).unwrap();
        }
    }
}

// This should not be needed generally
impl Subst {
    pub fn pretty_print(&self, out: &mut String) {
        write!(out, "{{{}}}", self.0).unwrap();
    }
}

impl NamePrefix {
    pub fn pretty_print(&self, out: &mut String) {
        match *self {
            NamePrefix::CrateId { ref name, ref dis } => {
                write!(out, "{}[{}]", name, dis).unwrap();
            }
            NamePrefix::TraitImpl { ref self_type, ref impled_trait } => {
                out.push('<');
                self_type.pretty_print(out);
                out.push_str(" as ");
                impled_trait.pretty_print(out);
                out.push('>');
            }
            NamePrefix::Node { ref prefix, ref ident } => {
                prefix.pretty_print(out);
                out.push_str("::");
                ident.pretty_print(out);
            }
            NamePrefix::Subst(subst) => {
                subst.pretty_print(out);
            }
        }
    }
}

impl NamePrefixWithParams {
    pub fn pretty_print(&self, out: &mut String) {
        match *self {
            NamePrefixWithParams::Node { ref prefix, ref args } => {
                prefix.pretty_print(out);
                args.pretty_print(out);
            }
            NamePrefixWithParams::Subst(subst) => {
                subst.pretty_print(out);
            }
        }
    }
}

impl FullyQualifiedName {
    pub fn pretty_print(&self, out: &mut String) {
        match *self {
            FullyQualifiedName::Name { ref name } => {
                name.pretty_print(out);
            }
            FullyQualifiedName::Subst(subst) => {
                subst.pretty_print(out);
            }
        }
    }
}

impl GenericArgumentList {

    pub fn pretty_print(&self, out: &mut String) {
        if self.params.len() > 0 {
            out.push('<');
            for param in &self.params {
                param.pretty_print(out);
                out.push(',');
            }
            out.pop();
            out.push('>');
        }
    }
}

impl Type {
    pub fn pretty_print(&self, out: &mut String) {
        match *self {
            Type::BasicType(t) => {
                t.pretty_print(out);
            }
            Type::Ref(ref t) => {
                out.push('&');
                t.pretty_print(out);
            }
            Type::RefMut(ref t) => {
                out.push_str("&mut ");
                t.pretty_print(out);
            }
            Type::RawPtrConst(ref t) => {
                out.push_str("*const ");
                t.pretty_print(out);
            }
            Type::RawPtrMut(ref t) => {
                out.push_str("*mut ");
                t.pretty_print(out);
            }
            Type::Array(opt_size, ref t) => {
                out.push('[');
                t.pretty_print(out);

                if let Some(size) = opt_size {
                    write!(out, "; {}", size).unwrap();
                }

                out.push(']');
            }
            Type::Tuple(ref components) => {
                out.push('(');
                for c in components {
                    c.pretty_print(out);
                    out.push(',');
                }
                out.pop();
                out.push(')');
            }
            Type::Named(ref qname) => {
                qname.pretty_print(out);
            }
            Type::GenericParam(ref name) => {
                out.push_str(name);
            }
            Type::Fn {
                ref return_type,
                ref params,
                is_unsafe,
                is_variadic,
                abi,
            } => {
                if is_unsafe {
                    out.push_str("unsafe ");
                }

                if abi != Abi::Rust {
                    out.push_str("extern ");
                    abi.pretty_print(out);
                    out.push(' ');
                }

                out.push_str("fn(");

                if params.len() > 0 || is_variadic {
                    for param in params {
                        param.pretty_print(out);
                        out.push(',');
                    }

                    if is_variadic {
                        out.push_str("...");
                    } else {
                        out.pop();
                    }
                }

                out.push(')');

                if **return_type != Type::BasicType(BasicType::Unit) {
                    out.push_str(" -> ");
                    return_type.pretty_print(out);
                }
            }
            Type::Subst(subst) => {
                subst.pretty_print(out);
            }
        }
    }
}

impl Abi {
    pub fn pretty_print(&self, out: &mut String) {
        out.push('"');
        match *self {
            Abi::Rust => {},
            Abi::C => out.push_str("C"),
        };
        out.push('"');
    }
}

impl BasicType {
    pub fn pretty_print(&self, out: &mut String) {
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
        });
    }
}

impl Symbol {
    pub fn pretty_print(&self, out: &mut String) {
        self.name.pretty_print(out);
    }
}
