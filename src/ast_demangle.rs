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

impl AstDemangle for Symbol {
    fn demangle_to_string(&self, out: &mut String) {
        self.path.demangle_to_string(out);

        if let Some(ref instantiating_crate) = self.instantiating_crate {
            out.push_str(" @ ");
            instantiating_crate.demangle_to_string(out);
        }
    }
}

impl AstDemangle for Ident {
    fn demangle_to_string(&self, out: &mut String) {

        self.u_ident.demangle_to_string(out);
        if self.dis != Base62Number(0) {
            write!(out, "[{}]", self.dis.0).unwrap();
        }
    }
}

impl AstDemangle for UIdent {
    fn demangle_to_string(&self, out: &mut String) {
        out.push_str(&self.0[..]);
    }
}

impl AstDemangle for Path {
    fn demangle_to_string(&self, out: &mut String) {
        match *self {
            Path::CrateRoot { ref id } => {
                id.demangle_to_string(out);
            }
            Path::InherentImpl { impl_path: _, ref self_type } => {
                out.push('<');
                self_type.demangle_to_string(out);
                out.push('>');
            }
            Path::TraitImpl { impl_path: _, ref self_type, ref trait_name } |
            Path::TraitDef { ref self_type, ref trait_name } => {
                out.push('<');
                self_type.demangle_to_string(out);
                out.push_str(" as ");
                trait_name.demangle_to_string(out);
                out.push('>');
            }
            Path::Nested { ref ns, ref inner, ref ident } => {
                inner.demangle_to_string(out);

                if *ns == Namespace(b'C') {
                    write!(out, "::{{closure}}[{}]", ident.dis.0).unwrap();
                } else if ident.u_ident.0.len() > 0 {
                    out.push_str("::");
                    ident.demangle_to_string(out);
                }
            }
            Path::Generic { ref inner, ref args } => {
                inner.demangle_to_string(out);
                out.push('<');
                for arg in args {
                    arg.demangle_to_string(out);
                    out.push(',');
                }
                out.pop();
                out.push('>');
            }
        }

    }
}

impl AstDemangle for DynBounds {
    fn demangle_to_string(&self, out: &mut String) {
        for tr in self.traits.iter() {
            tr.demangle_to_string(out);
            out.push_str("+");
        }

        out.pop();
    }
}

impl AstDemangle for GenericArg {
    fn demangle_to_string(&self, out: &mut String) {
        match *self {
            GenericArg::Lifetime(ref lt) => {
                lt.demangle_to_string(out);
            }
            GenericArg::Type(ref ty) => {
                ty.demangle_to_string(out);
            }
            GenericArg::Const(ref k) => {
                k.demangle_to_string(out);
            }
        }
    }
}

impl AstDemangle for Lifetime {
    fn demangle_to_string(&self, out: &mut String) {
        out.push_str("'_");
    }
}

impl AstDemangle for Type {
    fn demangle_to_string(&self, out: &mut String) {
        match *self {
            Type::BasicType(bt) => {
                bt.demangle_to_string(out);
            }
            Type::Array(ref inner, ref len) => {
                out.push_str("[");
                inner.demangle_to_string(out);
                out.push_str("; ");
                len.demangle_to_string(out);
                out.push_str("]");
            }
            Type::Slice(ref inner) => {
                out.push_str("[");
                inner.demangle_to_string(out);
                out.push_str("]");
            }
            Type::Named(ref path) => {
                path.demangle_to_string(out);
            }
            Type::Tuple(ref inner) => {
                out.push_str("(");
                for ty in inner {
                    ty.demangle_to_string(out);
                    out.push_str(",");
                }
                out.pop();
                out.push_str(")");
            }
            Type::Ref(_, ref ty) => {
                out.push_str("&");
                ty.demangle_to_string(out);
            }
            Type::RefMut(_, ref ty) => {
                out.push_str("&mut ");
                ty.demangle_to_string(out);
            }
            Type::RawPtrConst(ref ty)  => {
                out.push_str("*const ");
                ty.demangle_to_string(out);
            }
            Type::RawPtrMut(ref ty) => {
                out.push_str("*mut ");
                ty.demangle_to_string(out);
            }
            Type::Fn(ref fn_sig) => {
                fn_sig.demangle_to_string(out);
            }
            Type::DynTrait(ref bounds, _) => {
                bounds.demangle_to_string(out);
            }

        }
    }
}

impl AstDemangle for FnSig {
    fn demangle_to_string(&self, out: &mut String) {
        if self.is_unsafe {
            out.push_str("unsafe ");
        }

        if let Some(ref abi) = self.abi {
            out.push_str("extern ");
            abi.demangle_to_string(out);
            out.push_str(" ");
        }

        out.push_str("fn(");

        if self.param_types.len() > 0 {
            for param_type in self.param_types.iter() {
                param_type.demangle_to_string(out);
                out.push_str(",");
            }
            out.pop();
        }

        out.push_str(")");

        if self.return_type != Type::BasicType(BasicType::Unit) {
            out.push_str(" -> ");
            self.return_type.demangle_to_string(out);
        }
    }
}

impl AstDemangle for Abi {
    fn demangle_to_string(&self, out: &mut String) {
        out.push('"');
        match *self {
            Abi::C => {
                out.push('C');
            }
            Abi::Named(ref ident) => {
                ident.demangle_to_string(out);
            }
        }
        out.push('"');
    }
}


impl AstDemangle for DynTrait {
    fn demangle_to_string(&self, out: &mut String) {
        self.path.demangle_to_string(out);

        if self.assoc_type_bindings.len() > 0 {
            out.push('<');

            for binding in self.assoc_type_bindings.iter() {
                binding.demangle_to_string(out);
                out.push_str(", ");
            }

            out.pop();
            out.push('>');
        }

    }
}

impl AstDemangle for DynTraitAssocBinding {
    fn demangle_to_string(&self, out: &mut String) {
        self.ident.demangle_to_string(out);
        out.push('=');
        self.ty.demangle_to_string(out);
    }
}

impl AstDemangle for Const {
    fn demangle_to_string(&self, out: &mut String) {
        match *self {
            Const::Value(Type::BasicType(BasicType::I8), i) |
            Const::Value(Type::BasicType(BasicType::I16), i) |
            Const::Value(Type::BasicType(BasicType::I32), i) |
            Const::Value(Type::BasicType(BasicType::I64), i) |
            Const::Value(Type::BasicType(BasicType::I128), i) |
            Const::Value(Type::BasicType(BasicType::Isize), i) |
            Const::Value(Type::BasicType(BasicType::U8), i) |
            Const::Value(Type::BasicType(BasicType::U16), i) |
            Const::Value(Type::BasicType(BasicType::U32), i) |
            Const::Value(Type::BasicType(BasicType::U64), i) |
            Const::Value(Type::BasicType(BasicType::U128), i) |
            Const::Value(Type::BasicType(BasicType::Usize), i) => {
                write!(out, "{}", i).unwrap();
            }
            Const::Placeholder(ref ty) |
            Const::Value(ref ty, _) => {
                out.push_str("{const ");
                ty.demangle_to_string(out);
                out.push('}');
            }
        }
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
            BasicType::Placeholder => "_",
        });
    }
}
