use std::sync::Arc;

#[derive(Clone, PartialEq, Eq, Debug, Hash)]
pub enum IdentTag {
    ValueNs,
    TypeNs,
    Closure,
}

#[derive(Clone, PartialEq, Eq, Debug, Hash)]
pub struct Ident {
    pub ident: String,
    pub tag: IdentTag,
    pub dis: u32,
}

#[derive(Copy, Clone, PartialEq, Eq, Debug, Hash)]
pub struct Subst(pub usize);

#[derive(Clone, PartialEq, Eq, Debug, Hash)]
pub enum NamePrefix {
    CrateId {
        name: String,
        dis: String,
    },
    TraitImpl {
        self_type: Arc<Type>,
        impled_trait: Arc<FullyQualifiedName>,
        // TODO: bounds
    },
    Node {
        prefix: Arc<NamePrefixWithParams>,
        ident: Ident,
    },
    Subst(Subst)
}

#[derive(Clone, PartialEq, Eq, Debug, Hash)]
pub enum NamePrefixWithParams {
    Node {
        prefix: Arc<NamePrefix>,
        args: GenericArgumentList,
    },
    Subst(Subst),
}

#[derive(Clone, PartialEq, Eq, Debug, Hash)]
pub enum FullyQualifiedName {
    Name {
        name: Arc<NamePrefixWithParams>,
    },
    Subst(Subst),
}

#[derive(Clone, PartialEq, Eq, Debug, Hash)]
pub struct GenericArgumentList {
    pub params: Vec<Arc<Type>>,
}

#[derive(Clone, PartialEq, Eq, Debug, Hash)]
pub enum Type {
    BasicType(BasicType),
    Ref(Arc<Type>),
    RefMut(Arc<Type>),
    RawPtrConst(Arc<Type>),
    RawPtrMut(Arc<Type>),
    Array(Option<usize>, Arc<Type>),
    Tuple(Vec<Arc<Type>>),
    Named(Arc<FullyQualifiedName>),
    GenericParam(String), // Must support hygiene?
    Fn {
        is_unsafe: bool,
        is_variadic: bool,
        abi: Abi,
        return_type: Arc<Type>,
        params: Vec<Arc<Type>>,
    },
    Subst(Subst),
}

#[derive(Clone, PartialEq, Eq, Debug, Hash)]
pub struct ParamBound {
    pub param_name: String,
    pub bounds: Vec<Arc<Type>>,
}

#[derive(Copy, Clone, PartialEq, Eq, Debug, Hash)]
pub enum Abi {
    Rust,
    C,
    // TODO
}

#[derive(Copy, Clone, PartialEq, Eq, Debug, Hash)]
pub enum BasicType {
    Bool,
    Char,
    Str,
    Unit,
    I8,
    I16,
    I32,
    I64,
    I128,
    Isize,
    U8,
    U16,
    U32,
    U64,
    U128,
    Usize,
    F32,
    F64,
    Never,
}

#[derive(Clone, PartialEq, Eq, Debug, Hash)]
pub struct Symbol {
    pub name: Arc<FullyQualifiedName>,
    // pub instantiating_crate: Arc<NamePrefix>,
}

impl Ident {
    pub fn new(ident: &str, tag: IdentTag, dis: u32) -> Ident {
        Ident {
            ident: ident.into(),
            tag,
            dis,
        }
    }

    pub fn ty(ident: &str) -> Ident {
        Ident::new(ident, IdentTag::TypeNs, 0)
    }

    pub fn val(ident: &str) -> Ident {
        Ident::new(ident, IdentTag::ValueNs, 0)
    }

    pub fn closure(ident: &str) -> Ident {
        Ident::new(ident, IdentTag::Closure, 0)
    }
}

impl GenericArgumentList {
    pub fn new_empty() -> GenericArgumentList {
        GenericArgumentList {
            params: vec![],
        }
    }
}
