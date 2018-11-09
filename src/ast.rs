use std::ops::Deref;
use std::sync::Arc;

#[derive(Clone, PartialEq, Eq, Debug, Hash)]
pub enum IdentTag {
    Function,
    Static,
    TypeNs,
    Closure,
}

pub const NUMERIC_DISAMBIGUATOR_RADIX: u8 = 16;

#[derive(Copy, Clone, PartialEq, Eq, Debug, Hash)]
pub struct NumericDisambiguator(pub u64);

#[derive(Clone, PartialEq, Eq, Debug, Hash)]
pub struct Ident {
    pub ident: String,
    pub tag: IdentTag,
    pub dis: NumericDisambiguator,
}

pub const SUBST_RADIX: u8 = 16;

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug, Hash)]
pub struct Subst(pub u64);

#[derive(Clone, PartialEq, Eq, Debug, Hash)]
pub enum NamePrefix {
    CrateId {
        name: String,
        dis: String,
    },
    TraitImpl {
        self_type: Arc<Type>,
        impled_trait: Arc<QName>,
        dis: NumericDisambiguator,
    },
    InherentImpl {
        self_type: Arc<Type>,
    },
    Node {
        prefix: Arc<NamePrefix>,
        ident: Ident,
    },
    Subst(Subst),
}

#[derive(Clone, PartialEq, Eq, Debug, Hash)]
pub enum QName {
    Name {
        name: Arc<NamePrefix>,
        args: GenericArgumentList,
    },
    Subst(Subst),
}

#[derive(Clone, PartialEq, Eq, Debug, Hash)]
pub struct GenericArgumentList(pub Vec<Arc<Type>>);

#[derive(Clone, PartialEq, Eq, Debug, Hash)]
pub enum Type {
    BasicType(BasicType),
    Ref(Arc<Type>),
    RefMut(Arc<Type>),
    RawPtrConst(Arc<Type>),
    RawPtrMut(Arc<Type>),
    Array(Option<u64>, Arc<Type>),
    Tuple(Vec<Arc<Type>>),
    Named(Arc<QName>),
    GenericParam(Ident),
    Fn {
        is_unsafe: bool,
        abi: Abi,
        params: Vec<Arc<Type>>,
        return_type: Option<Arc<Type>>,
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
    Ellipsis,
}

#[derive(Clone, PartialEq, Eq, Debug, Hash)]
pub struct Symbol {
    pub name: Arc<QName>,
    pub instantiating_crate: Option<Arc<NamePrefix>>,
}

impl GenericArgumentList {
    pub fn new_empty() -> GenericArgumentList {
        GenericArgumentList(vec![])
    }

    pub fn ptr_eq(&self, other: &GenericArgumentList) -> bool {
        assert_eq!(self.len(), other.len());

        self.iter()
            .zip(other.iter())
            .all(|(a, b)| Arc::ptr_eq(a, b))
    }
}

impl Deref for GenericArgumentList {
    type Target = [Arc<Type>];

    fn deref(&self) -> &[Arc<Type>] {
        &self.0[..]
    }
}
