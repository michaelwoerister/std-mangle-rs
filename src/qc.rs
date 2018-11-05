

use std::sync::Arc;
use quickcheck::{Gen, Arbitrary, StdThreadGen};
use ast::*;
use unicode_xid::UnicodeXID;
use rand::Rng;

use std::cmp;

impl Arbitrary for IdentTag {
    fn arbitrary<G: Gen>(g: &mut G) -> Self {
        match g.next_u32() % 4 {
            0 => IdentTag::Function,
            1 => IdentTag::Static,
            2 => IdentTag::TypeNs,
            3 => IdentTag::Closure,
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
                let len = if len >= 5000 {
                    None
                } else {
                    Some(len)
                };

                Type::Array(len, Arbitrary::arbitrary(g))
            }
            6 => {
                let components = loop {
                    let components: Vec<Arc<Type>> = Arbitrary::arbitrary(g);
                    if components.len() > 0 {
                        break components;
                    }
                };

                Type::Tuple(components)
            }
            7 => Type::Named(Arc::new(Arbitrary::arbitrary(g))),
            8 => Type::GenericParam(gen_valid_ident(g)),
            9 => {
                Type::Fn {
                    is_unsafe: Arbitrary::arbitrary(g),
                    abi: Arbitrary::arbitrary(g),
                    params: Arbitrary::arbitrary(g),
                    return_type: Arbitrary::arbitrary(g),
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

impl Arbitrary for NamePrefix {
    fn arbitrary<G: Gen>(g: &mut G) -> Self {

        let size = { let s = g.size(); g.gen_range(0, s) };

        let mut node = NamePrefix::CrateId {
            name: gen_valid_ident(g),
            dis: "abc".to_string(), // TODO
        };

        for _ in 0 .. size {
            node = NamePrefix::Node {
                prefix: Arc::new(node),
                ident: Arbitrary::arbitrary(g),
            };
        }

        node
    }
}

impl Arbitrary for QName {
    fn arbitrary<G: Gen>(g: &mut G) -> Self {
        QName::Name {
            name: Arbitrary::arbitrary(g),
            args: Arbitrary::arbitrary(g),
        }
    }
}

impl Arbitrary for Symbol {
    fn arbitrary<G: Gen>(g: &mut G) -> Self {
        Symbol {
            name: Arbitrary::arbitrary(g),
            // instantiating_crate: Arc::new(NamePrefix::CrateId {
            //         name: gen_valid_ident(g),
            //         dis: "abc".to_string(), // TODO
            // }),
        }
    }
}

// const ASCII_ONLY: bool = false;
// const ASCII: &[u8] = b"0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ_";
// const START_SET_START: usize = 10;

// fn gen_valid_ident<G: Gen>(g: &mut G) -> String {

//     let len = (1 + g.next_u32() % 20) as usize;
//     let mut s = String::with_capacity(len);

//     let ascii_only = ASCII_ONLY || (g.next_u32() % 5 != 0);
//     if ascii_only {
//         s.push(*g.choose(&ASCII[START_SET_START ..]).unwrap() as char);
//         assert!(!s.as_bytes()[0].is_ascii_digit());

//         for _ in 0 .. len {
//             s.push(*g.choose(ASCII).unwrap() as char);
//         }
//     } else {
//         let start = loop {
//             let c: char = Arbitrary::arbitrary(g);
//             if UnicodeXID::is_xid_start(c) {
//                 break c
//             }
//         };

//         s.push(start);

//         while s.len() < len {
//             let c: char = Arbitrary::arbitrary(g);
//             if UnicodeXID::is_xid_continue(c) {
//                 s.push(c);
//             }
//         }
//     }

//     s
// }

const VALID_IDENTS: &[&str] = &[
    "foo",
    "_foo",
    "f00",
    "_0",
    "__1",
    "foo_",
    "E",
    "N",
    "X",
    "S",
];

fn gen_valid_ident<G: Gen>(g: &mut G) -> String {
    g.choose(VALID_IDENTS).unwrap().to_string()
}


