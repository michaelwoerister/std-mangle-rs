use std::sync::Arc;

#[derive(Copy, Clone, PartialEq, Eq, Debug, Hash)]
pub struct Base62Number(pub u64);

#[derive(Copy, Clone, PartialEq, Eq, Debug, Hash)]
pub struct DecimalNumber(pub u64);

#[derive(Clone, PartialEq, Eq, Debug, Hash)]
pub struct Ident {
    pub dis: Base62Number,
    pub u_ident: UIdent,
}

#[derive(Clone, PartialEq, Eq, Debug, Hash)]
pub struct UIdent(pub String);

#[derive(Clone, PartialEq, Eq, Debug, Hash)]
pub struct Namespace(pub u8);

#[derive(Clone, PartialEq, Eq, Debug, Hash)]
pub enum Path {
    CrateRoot {
        id: Ident,
    },
    InherentImpl {
        impl_path: ImplPath,
        self_type: Type,
    },
    TraitImpl {
        impl_path: ImplPath,
        self_type: Type,
        trait_name: Arc<Path>,
    },
    TraitDef {
        self_type: Type,
        trait_name: Arc<Path>,
    },
    Nested {
        ns: Namespace,
        inner: Arc<Path>,
        ident: Ident,
    },
    Generic {
        inner: Arc<Path>,
        args: Vec<GenericArg>,
    },
}

#[derive(Clone, PartialEq, Eq, Debug, Hash)]
pub struct ImplPath {
    // Should this be optional?
    pub dis: Option<Base62Number>,
    pub path: Arc<Path>,
}

#[derive(Clone, PartialEq, Eq, Debug, Hash)]
pub enum GenericArg {
    Lifetime(Lifetime),
    Type(Type),
    Const(Const),
}

#[derive(Clone, PartialEq, Eq, Debug, Hash)]
pub struct Lifetime {
    pub debruijn_index: Base62Number,
}

#[derive(Clone, PartialEq, Eq, Debug, Hash)]
pub struct Binder {
    pub count: Base62Number,
}

#[derive(Clone, PartialEq, Eq, Debug, Hash)]
pub enum Type {
    BasicType(BasicType),
    Array(Arc<Type>, Arc<Const>),
    Slice(Arc<Type>),
    Named(Arc<Path>),
    Tuple(Vec<Type>),
    Ref(Option<Lifetime>, Arc<Type>),
    RefMut(Option<Lifetime>, Arc<Type>),
    RawPtrConst(Arc<Type>),
    RawPtrMut(Arc<Type>),
    Fn(Arc<FnSig>),
    DynTrait(Arc<DynBounds>, Lifetime),
}

#[derive(Clone, PartialEq, Eq, Debug, Hash)]
pub struct FnSig {
    pub binder: Binder,
    pub is_unsafe: bool,
    pub abi: Option<Abi>,
    pub param_types: Vec<Type>,
    pub return_type: Type,
}

#[derive(Clone, PartialEq, Eq, Debug, Hash)]
pub enum Abi {
    C,
    Named(UIdent),
}

#[derive(Clone, PartialEq, Eq, Debug, Hash)]
pub struct DynBounds {
    pub binder: Binder,
    pub traits: Vec<DynTrait>,
}

#[derive(Clone, PartialEq, Eq, Debug, Hash)]
pub struct DynTrait {
    pub path: Path,
    pub assoc_type_bindings: Vec<DynTraitAssocBinding>,
}

#[derive(Clone, PartialEq, Eq, Debug, Hash)]
pub struct DynTraitAssocBinding {
    pub ident: UIdent,
    pub ty: Type,
}

#[derive(Clone, PartialEq, Eq, Debug, Hash)]
pub enum Const {
    Value(Type, u64),
    Placeholder(Type),
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
    Ellipsis,
    Placeholder,
}

#[derive(Clone, PartialEq, Eq, Debug, Hash)]
pub struct Symbol {
    pub version: Option<DecimalNumber>,
    pub path: Path,
    pub instantiating_crate: Option<Path>,
}
