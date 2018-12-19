use ast::*;
use quickcheck::{Arbitrary, Gen, StdThreadGen};
use rand::Rng;
use std::cmp;
use std::sync::Arc;

impl Arbitrary for IdentTag {
    fn arbitrary<G: Gen>(g: &mut G) -> Self {
        match g.next_u32() % 3 {
            0 => IdentTag::TypeNs,
            1 => IdentTag::ValueNs,
            2 => IdentTag::Closure,
            _ => unreachable!(),
        }
    }
}

impl Arbitrary for NumericDisambiguator {
    fn arbitrary<G: Gen>(g: &mut G) -> Self {
        match g.next_u32() % 2 {
            0 => NumericDisambiguator(g.next_u32() as u64 % 9),
            1 => NumericDisambiguator(100 + g.next_u32() as u64 % 100),
            _ => unreachable!(),
        }
    }
}

impl Arbitrary for GenericArgumentList {
    fn arbitrary<G: Gen>(g: &mut G) -> Self {
        let size = cmp::min(6, g.size()) / 2;

        if size > 0 {
            let g = &mut StdThreadGen::new(size);
            GenericArgumentList(Arbitrary::arbitrary(g))
        } else {
            GenericArgumentList(vec![])
        }
    }
}

impl Arbitrary for Type {
    fn arbitrary<G: Gen>(g: &mut G) -> Self {
        match g.next_u32() % 10 {
            0 => Type::BasicType(Arbitrary::arbitrary(g)),
            1 => Type::Ref(Arc::new(Arbitrary::arbitrary(g))),
            2 => Type::RefMut(Arc::new(Arbitrary::arbitrary(g))),
            3 => Type::RawPtrConst(Arc::new(Arbitrary::arbitrary(g))),
            4 => Type::RawPtrMut(Arc::new(Arbitrary::arbitrary(g))),
            5 => {
                let len = (g.next_u32() % 10000) as u64;
                let len = if len >= 5000 { None } else { Some(len) };

                Type::Array(len, Arbitrary::arbitrary(g))
            }
            6 => {
                let len = g.gen_range(2, 4);
                let mut components: Vec<Arc<Type>> = Vec::with_capacity(len);

                for _ in 0..len {
                    components.push(Arbitrary::arbitrary(g));
                }

                Type::Tuple(components)
            }
            7 => Type::Named(Arc::new(Arbitrary::arbitrary(g))),
            8 => Type::GenericParam(generate_ident(g, Charset::Unicode, 1000)),
            9 => Type::Fn {
                is_unsafe: Arbitrary::arbitrary(g),
                abi: Arbitrary::arbitrary(g),
                params: Arbitrary::arbitrary(g),
                return_type: Arbitrary::arbitrary(g),
            },
            _ => unreachable!(),
        }
    }
}

impl Arbitrary for Abi {
    fn arbitrary<G: Gen>(g: &mut G) -> Self {
        match g.next_u32() % 2 {
            0 => Abi::Rust,
            1 => Abi::C,

            _ => unreachable!(),
        }
    }
}

impl Arbitrary for BasicType {
    fn arbitrary<G: Gen>(g: &mut G) -> Self {
        match g.next_u32() % 19 {
            0 => BasicType::Bool,
            1 => BasicType::Char,
            2 => BasicType::Str,
            3 => BasicType::Unit,
            4 => BasicType::I8,
            5 => BasicType::I16,
            6 => BasicType::I32,
            7 => BasicType::I64,
            8 => BasicType::I128,
            9 => BasicType::Isize,
            10 => BasicType::U8,
            11 => BasicType::U16,
            12 => BasicType::U32,
            13 => BasicType::U64,
            14 => BasicType::U128,
            15 => BasicType::Usize,
            16 => BasicType::F32,
            17 => BasicType::F64,
            18 => BasicType::Never,
            _ => unreachable!(),
        }
    }
}

#[derive(Copy, Clone)]
enum Charset {
    Ascii,
    Unicode,
}

impl Arbitrary for AbsolutePath {
    fn arbitrary<G: Gen>(g: &mut G) -> Self {
        let len = {
            let s = g.size();

            if s > 1 {
                1 + g.gen_range(0, s)
            } else {
                1
            }
        };

        let charset = *g.choose(&[Charset::Ascii, Charset::Unicode]).unwrap();

        let non_crate_root = g.size() > 2;

        let mut path = Arc::new(match (g.gen_range(0, 100), non_crate_root) {
            (0, true) => {
                let mut smaller_rng = get_smaller_rng(g);

                PathPrefix::TraitImpl {
                    self_type: Arbitrary::arbitrary(&mut smaller_rng),
                    impled_trait: Arbitrary::arbitrary(&mut smaller_rng),
                    dis: Arbitrary::arbitrary(&mut smaller_rng),
                }
            }
            _ => PathPrefix::CrateId {
                name: generate_ident(g, charset, 2).ident,
                dis: generate_crate_disambiguator(g),
            },
        });

        for i in 0..len {
            let max = 2 * (i + 1);

            path = Arc::new(PathPrefix::Node {
                prefix: path,
                ident: generate_ident(g, charset, max),
            });
        }

        AbsolutePath::Path {
            name: path,
            args: Arbitrary::arbitrary(g),
        }
    }
}

impl Arbitrary for Symbol {
    fn arbitrary<G: Gen>(g: &mut G) -> Self {
        Symbol {
            name: Arbitrary::arbitrary(g),
            instantiating_crate: {
                if g.next_u32() % 3 == 0 {
                    Some(Arc::new(PathPrefix::CrateId {
                        name: generate_ident(g, Charset::Ascii, 3).ident,
                        dis: generate_crate_disambiguator(g),
                    }))
                } else {
                    None
                }
            },
        }
    }
}

const IDENTS_ASCII: &[&str] = &[
    "foo", "_foo", "f00", "_0", "foo_", "E", "N", "S", "foo", "_foo", "f00", "_0", "foo_", "E",
    "N", "S",
];

const IDENTS_UNICODE: &[&str] = &[
    "foo",
    "ρυστ",
    "_foo",
    "铁锈",
    "f00",
    "Schrödinger",
    "_0",
    "𝔸𝔸𝔸",
    "foo_",
    "😊",
    "E",
    "⚘",
    "N",
    "∀",
    "S",
    "🤦",
];

fn generate_ident<G: Gen>(g: &mut G, kind: Charset, max: usize) -> Ident {
    let idents = match kind {
        Charset::Ascii => IDENTS_ASCII,
        Charset::Unicode => IDENTS_UNICODE,
    };

    let tag = Arbitrary::arbitrary(g);

    let ident = if tag == IdentTag::Closure {
        String::new()
    } else {
        let index = (g.next_u32() as usize) % cmp::min(idents.len(), max);
        idents[index].to_string()
    };

    Ident {
        ident,
        tag,
        dis: Arbitrary::arbitrary(g),
    }
}

fn generate_crate_disambiguator<G: Gen>(_: &mut G) -> String {
    // We actually don't want variation here, otherwise things won't be
    // compressible at all.
    "abc".to_string()
}

fn get_smaller_rng<G: Gen>(g: &G) -> StdThreadGen {
    let size = cmp::min(6, g.size()) / 2;
    StdThreadGen::new(size)
}
