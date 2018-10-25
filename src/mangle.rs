use ast::*;
use std::fmt::Write;

impl IdentTag {
    pub fn mangle(&self, out: &mut String) {
        match *self {
            IdentTag::TypeNs => {},
            IdentTag::ValueNs => out.push('V'),
            IdentTag::Closure => out.push('C'),
        }
    }
}


impl Ident {
    pub fn mangle(&self, out: &mut String) {
        let len = self.ident.len();
        write!(out, "{}{}", len, self.ident).unwrap();

        self.tag.mangle(out);

        if self.dis != 0 {
            write!(out, "s{}_", self.dis - 1).unwrap();
        }
    }
}

impl Subst {
    pub fn mangle(&self, out: &mut String) {
        if self.0 == 0 {
            out.push_str("S_");
        } else {
            write!(out, "S{}_", self.0 - 1).unwrap();
        }
    }
}

impl NamePrefix {
    pub fn mangle(&self, out: &mut String) {
        match *self {
            NamePrefix::CrateId { ref name, ref dis } => {
                let len = name.len() + dis.len() + 1;
                write!(out, "N{}{}_{}", len, name, dis).unwrap();
            }
            NamePrefix::TraitImpl { ref self_type, ref impled_trait } => {
                out.push('X');
                self_type.mangle(out);
                impled_trait.mangle(out);
            }
            NamePrefix::Node { ref prefix, ref ident } => {
                prefix.mangle(out);
                ident.mangle(out);
            }
            NamePrefix::Subst(subst) => {
                subst.mangle(out);
            }
        }
    }
}

impl NamePrefixWithParams {
    pub fn mangle(&self, out: &mut String) {
        match *self {
            NamePrefixWithParams::Node { ref prefix, ref args } => {
                prefix.mangle(out);
                args.mangle(out);
            }
            NamePrefixWithParams::Subst(subst) => {
                subst.mangle(out);
            }
        }
    }
}


impl FullyQualifiedName {
    pub fn mangle(&self, out: &mut String) {
        match *self {
            FullyQualifiedName::Name { ref name } => {
                name.mangle(out);
                out.push('E');
            }
            FullyQualifiedName::Subst(subst) => {
                subst.mangle(out);
            }
        }
    }
}

impl GenericArgumentList {

    pub fn mangle(&self, out: &mut String) {
        out.push('I');
        for param in &self.params {
            param.mangle(out);
        }
        out.push('E');
        // TODO: bounds
    }
}

impl Type {
    pub fn mangle(&self, out: &mut String) {
        match *self {
            Type::BasicType(t) => {
                t.mangle(out);
            }
            Type::Ref(ref t) => {
                out.push('R'); // TODO
                t.mangle(out);
            }
            Type::RefMut(ref t) => {
                out.push('Q'); // TODO
                t.mangle(out);
            }
            Type::RawPtrConst(ref t) => {
                out.push('P'); // TODO
                t.mangle(out);
            }
            Type::RawPtrMut(ref t) => {
                out.push('K'); // TODO
                t.mangle(out);
            }
            Type::Tuple(ref components) => {
                out.push('T');
                for c in components {
                    c.mangle(out);
                }
                out.push('E');
            }
            Type::Named(ref qname) => {
                qname.mangle(out);
            }
            Type::GenericParam(ref name) => {
                write!(out, "{}{}", name.len(), name).unwrap();
            }
            Type::Fn {
                ref return_type,
                ref params,
                is_unsafe,
                is_variadic,
                abi,
            } => {
                out.push('F');

                if is_unsafe {
                    out.push('U');
                }

                if is_variadic {
                    out.push('V');
                }

                abi.mangle(out);

                return_type.mangle(out);

                for param in params {
                    param.mangle(out);
                }

                out.push('E');
            }
            Type::Subst(subst) => {
                subst.mangle(out);
            }
        }
    }
}

// TODO
// #[derive(Clone, PartialEq, Eq, Debug, Hash)]
// pub struct ParamBound {
//     pub param_name: String,
//     pub bounds: Vec<Arc<Type>>,
// }

// TODO: assignment
impl Abi {
    pub fn mangle(&self, out: &mut String) {
        match *self {
            Abi::Rust => {},
            Abi::C => out.push('C'),
        };
    }
}

impl BasicType {
    pub fn mangle(&self, out: &mut String) {
        out.push(match *self {
            BasicType::Bool => 'x',
            BasicType::Char => 'x',
            BasicType::Str => 'x',
            BasicType::Unit => 'x',
            BasicType::I8 => 'x',
            BasicType::I16 => 'x',
            BasicType::I32 => 'x',
            BasicType::I64 => 'x',
            BasicType::I128 => 'x',
            BasicType::Isize => 'x',
            BasicType::U8 => 'x',
            BasicType::U16 => 'x',
            BasicType::U32 => 'x',
            BasicType::U64 => 'x',
            BasicType::U128 => 'x',
            BasicType::Usize => 'x',
            BasicType::F32 => 'x',
            BasicType::F64 => 'x',
            BasicType::Never => 'x',
        });
    }
}

impl Symbol {
    pub fn mangle(&self, out: &mut String) {
        out.push_str("_R");
        self.name.mangle(out);
        self.instantiating_crate.mangle(out);
    }
}
