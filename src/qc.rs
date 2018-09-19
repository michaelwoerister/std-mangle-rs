

use std::sync::Arc;
use quickcheck::{Gen, Arbitrary, StdThreadGen};
use ast::*;
use unicode_xid::UnicodeXID;
use rand::Rng;

use std::cmp;

impl Arbitrary for IdentTag {
    fn arbitrary<G: Gen>(g: &mut G) -> Self {
        match g.next_u32() % 4 {
            0 => IdentTag::ValueNs,
            1 => IdentTag::TypeNs,
            2 => IdentTag::Closure,
            3 => IdentTag::Trait,
            _ => unreachable!(),
        }
    }
}

impl Arbitrary for Ident {
    fn arbitrary<G: Gen>(g: &mut G) -> Self {

        Ident {
            ident: gen_valid_ident(g),
            tag: Arbitrary::arbitrary(g),
            dis: Arbitrary::arbitrary(g),
        }
    }
}

impl Arbitrary for GenericArgumentList {
    fn arbitrary<G: Gen>(g: &mut G) -> Self {

        let size = cmp::min(6, g.size()) / 2;

        if size > 0 {
            let g = &mut StdThreadGen::new(size);
            GenericArgumentList {
                params: Arbitrary::arbitrary(g),
                bounds: Arbitrary::arbitrary(g),
            }
        } else {
            GenericArgumentList {
                params: vec![],
                bounds: vec![],
            }
        }
    }
}

impl Arbitrary for Type {
    fn arbitrary<G: Gen>(g: &mut G) -> Self {
        match g.next_u32() % 9 {
            0 => Type::BasicType(Arbitrary::arbitrary(g)),
            1 => Type::Ref(Arc::new(Arbitrary::arbitrary(g))),
            2 => Type::RefMut(Arc::new(Arbitrary::arbitrary(g))),
            3 => Type::RawPtrConst(Arc::new(Arbitrary::arbitrary(g))),
            4 => Type::RawPtrMut(Arc::new(Arbitrary::arbitrary(g))),
            5 => Type::Tuple(Arbitrary::arbitrary(g)),
            6 => Type::Named(Arc::new(Arbitrary::arbitrary(g))),
            7 => Type::GenericParam(gen_valid_ident(g)),
            8 => {
                Type::Fn {
                    return_type: Arc::new(Arbitrary::arbitrary(g)),
                    params: Arbitrary::arbitrary(g),
                    is_unsafe: Arbitrary::arbitrary(g),
                    is_variadic: Arbitrary::arbitrary(g),
                    abi: Arbitrary::arbitrary(g),
                }
            }
            _ => unreachable!()
        }
    }
}


impl Arbitrary for ParamBound {
    fn arbitrary<G: Gen>(g: &mut G) -> Self {
        ParamBound {
            param_name: gen_valid_ident(g),
            bounds: Arbitrary::arbitrary(g),
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

impl Arbitrary for NamePrefixWithParams {
    fn arbitrary<G: Gen>(g: &mut G) -> Self {

        let size = { let s = g.size(); g.gen_range(0, s) };

        let mut node = NamePrefixWithParams::Node {
            prefix: Arc::new(NamePrefix::CrateId {
                    name: gen_valid_ident(g),
                    dis: "abc".to_string(), // TODO
            }),
            args: GenericArgumentList {
                params: vec![],
                bounds: vec![],
            }
        } ;

        for _ in 0 .. size {
            let prefix = NamePrefix::Node {
                prefix: Arc::new(node),
                ident: Arbitrary::arbitrary(g),
            };

            node = NamePrefixWithParams::Node {
                prefix: Arc::new(prefix),
                args: Arbitrary::arbitrary(g),
            };
        }

        node
    }
}

impl Arbitrary for Symbol {
    fn arbitrary<G: Gen>(g: &mut G) -> Self {
        Symbol {
            name: Arbitrary::arbitrary(g),
            instantiating_crate: Arc::new(NamePrefix::CrateId {
                    name: gen_valid_ident(g),
                    dis: "abc".to_string(), // TODO
            }),
        }
    }
}

fn gen_valid_ident<G: Gen>(g: &mut G) -> String {
    let start = loop {
        let c: char = Arbitrary::arbitrary(g);
        if UnicodeXID::is_xid_start(c) {
            break c
        }
    };

    let len = (1 + g.next_u32() % 20) as usize;
    let mut s = String::with_capacity(len);
    s.push(start);

    while s.len() < len {
        let c: char = Arbitrary::arbitrary(g);
        if UnicodeXID::is_xid_continue(c) {
            s.push(c);
        }
    }

    s
}
