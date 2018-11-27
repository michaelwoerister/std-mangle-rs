use ast::*;
use charset;
use std::fmt::Write;

impl IdentTag {
    pub fn mangle(&self, out: &mut String) {
        match *self {
            IdentTag::TypeNs => {}
            IdentTag::ValueNs => out.push('V'),
            IdentTag::Closure => out.push('C'),
        }
    }
}

impl NumericDisambiguator {
    pub fn mangle(&self, out: &mut String) {
        let NumericDisambiguator(index) = *self;

        match index {
            0 => {
                // Don't print anything
            }
            1 => {
                // Don't print an index
                out.push_str("s_");
            }
            index => {
                write!(out, "s{:x}_", index - 2).unwrap();
            }
        }
    }
}

impl Ident {
    pub fn mangle(&self, out: &mut String) {
        charset::write_len_prefixed_ident(&[&self.ident], out).unwrap();
        self.tag.mangle(out);
        self.dis.mangle(out);
    }
}

impl Subst {
    pub fn mangle(&self, out: &mut String) {
        if self.0 == 0 {
            out.push_str("S_");
        } else {
            write!(out, "S{:x}_", self.0 - 1).unwrap();
        }
    }
}

impl PathPrefix {
    pub fn mangle(&self, out: &mut String) {
        match *self {
            PathPrefix::CrateId { ref name, ref dis } => {
                charset::write_len_prefixed_ident(&[name, "_", dis], out).unwrap();
            }
            PathPrefix::TraitImpl {
                ref self_type,
                ref impled_trait,
                dis,
            } => {
                out.push('X');
                self_type.mangle(out);
                impled_trait.mangle(out);
                dis.mangle(out);
            }
            PathPrefix::InherentImpl { ref self_type } => {
                out.push('M');
                self_type.mangle(out);
            }
            PathPrefix::Node {
                ref prefix,
                ref ident,
            } => {
                prefix.mangle(out);
                ident.mangle(out);
            }
            PathPrefix::Subst(subst) => {
                subst.mangle(out);
            }
        }
    }
}

impl AbsolutePath {
    pub fn mangle(&self, out: &mut String) {
        match *self {
            AbsolutePath::Path { ref name, ref args } => {
                out.push('N');
                name.mangle(out);
                args.mangle(out);
                out.push('E');
            }
            AbsolutePath::Subst(subst) => {
                subst.mangle(out);
            }
        }
    }
}

impl GenericArgumentList {
    pub fn mangle(&self, out: &mut String) {
        if self.len() > 0 {
            out.push('I');
            for param in self.iter() {
                param.mangle(out);
            }
            out.push('E');
        }
    }
}

impl Type {
    pub fn mangle(&self, out: &mut String) {
        match *self {
            Type::BasicType(t) => {
                t.mangle(out);
            }
            Type::Ref(ref t) => {
                out.push('R');
                t.mangle(out);
            }
            Type::RefMut(ref t) => {
                out.push('Q');
                t.mangle(out);
            }
            Type::RawPtrConst(ref t) => {
                out.push('P');
                t.mangle(out);
            }
            Type::RawPtrMut(ref t) => {
                out.push('O');
                t.mangle(out);
            }
            Type::Array(opt_size, ref t) => {
                out.push('A');
                if let Some(size) = opt_size {
                    write!(out, "{}", size).unwrap();
                }
                t.mangle(out);
            }
            Type::Tuple(ref components) => {
                out.push('T');
                for c in components {
                    c.mangle(out);
                }
                out.push('E');
            }
            Type::Named(ref abs_path) => {
                abs_path.mangle(out);
            }
            Type::GenericParam(ref ident) => {
                out.push('G');
                ident.mangle(out);
                out.push('E');
            }
            Type::Fn {
                ref return_type,
                ref params,
                is_unsafe,
                abi,
            } => {
                out.push('F');

                if is_unsafe {
                    out.push('U');
                }

                abi.mangle(out);

                for param in params {
                    param.mangle(out);
                }

                if let &Some(ref return_type) = return_type {
                    out.push('J');
                    return_type.mangle(out);
                }

                out.push('E');
            }
            Type::Subst(subst) => {
                subst.mangle(out);
            }
        }
    }
}

// TODO: assignment
impl Abi {
    pub fn mangle(&self, out: &mut String) {
        match *self {
            Abi::Rust => {}
            Abi::C => out.push_str("Kc"),
        };
    }
}

impl BasicType {
    pub fn mangle(&self, out: &mut String) {
        out.push(match *self {
            BasicType::I8 => 'a',
            BasicType::Bool => 'b',
            BasicType::Char => 'c',
            BasicType::F64 => 'd',
            BasicType::Str => 'e',
            BasicType::F32 => 'f',
            BasicType::U8 => 'h',
            BasicType::Isize => 'i',
            BasicType::Usize => 'j',
            BasicType::I32 => 'l',
            BasicType::U32 => 'm',
            BasicType::I128 => 'n',
            BasicType::U128 => 'o',
            BasicType::I16 => 's',
            BasicType::U16 => 't',
            BasicType::Unit => 'u',
            BasicType::Ellipsis => 'v',
            BasicType::I64 => 'x',
            BasicType::U64 => 'y',
            BasicType::Never => 'z',
        });
    }
}

impl Symbol {
    pub fn mangle(&self, out: &mut String) {
        out.push_str("_R");
        self.name.mangle(out);

        if let Some(ref instantiating_crate) = self.instantiating_crate {
            instantiating_crate.mangle(out);
        }
    }
}
