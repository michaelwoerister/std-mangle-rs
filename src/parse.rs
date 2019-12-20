use ast::*;
use charset;
use error::{self, expected};
use int_radix::ascii_digit_to_value;
use std::str;
use std::sync::Arc;

pub const EOT: u8 = 5; // ASCII "end of transmission"


pub fn parse(input: &[u8]) -> Result<Symbol, String> {
    let mut parser = Parser {
        input,
        pos: 0,
    };

    parser.parse_symbol()
          .map_err(|e| format!("at position {}: {}", parser.pos, e))
}

pub struct Parser<'input> {
    input: &'input [u8],
    pos: usize,
}

impl<'input> Parser<'input> {

    fn parse_symbol(&mut self) -> Result<Symbol, String> {

        if &self.input[0 .. 2] != b"_R" {
            return Err(format!("Not a Rust symbol"));
        }

        self.pos += 2;

        let version = if self.cur().is_ascii_digit() {
            let encoding_version = self.parse_number(10)? + 1;
            return error::version_mismatch(encoding_version, 0);
        } else {
            None
        };

        let path = self.parse_path()?;

        let instantiating_crate = if self.cur() != EOT {
            Some(self.parse_path()?)
        } else {
            None
        };

        Ok(Symbol {
            version,
            path,
            instantiating_crate,
        })
    }

    fn parse_const(&mut self) -> Result<Const, String> {
        if self.cur() == b'B' {
            let mut parser = self.parse_backref()?;
            parser.parse_const()
        } else {
            let ty = self.parse_type()?;

            if self.cur() == b'p' {
                Ok(Const::Placeholder(ty))
            } else {
                let value = self.parse_number(16)?;
                self.eat(b'_', "<const-data>")?;
                Ok(Const::Value(ty, value))
            }
        }
    }

    fn parse_generic_arg(&mut self) -> Result<GenericArg, String> {
        Ok(match self.cur() {
            b'L' => {
                GenericArg::Lifetime(self.parse_lifetime()?)
            }
            b'K' => {
                GenericArg::Const(self.parse_const()?)
            }
            _ => {
                GenericArg::Type(self.parse_type()?)
            }
        })
    }

    fn parse_lifetime(&mut self) -> Result<Lifetime, String> {
        self.eat(b'L', "<lifetime>")?;
        Ok(Lifetime {
            debruijn_index: self.parse_base62_number()?,
        })
    }

    fn parse_binder(&mut self) -> Result<Binder, String> {
        self.eat(b'G', "<binder>")?;

        Ok(Binder {
            count: self.parse_base62_number()?,
        })
    }

    fn parse_abi(&mut self) -> Result<Abi, String> {
        if self.cur() == b'C' {
            self.pos += 1;
            Ok(Abi::C)
        } else {
            Ok(Abi::Named(self.parse_uident()?))
        }
    }

    fn parse_fn_sig(&mut self) -> Result<FnSig, String> {
        let binder = self.parse_binder()?;
        let is_unsafe = self.try_eat(b'U');
        let abi = if self.try_eat(b'K') {
            Some(self.parse_abi()?)
        } else {
            None
        };

        let mut param_types = Vec::new();

        while self.cur() != b'E' {
            param_types.push(self.parse_type()?);
        }

        self.eat(b'E', "<fn-sig>")?;

        let return_type = self.parse_type()?;

        Ok(FnSig {
            binder,
            is_unsafe,
            abi,
            param_types,
            return_type
        })
    }

    fn parse_dyn_bounds(&mut self) -> Result<DynBounds, String> {
        let binder = self.parse_binder()?;
        let mut traits = Vec::new();
        while self.cur() != b'E' {
            traits.push(self.parse_dyn_trait()?);
        }
        self.eat(b'E', "<dyn-trait>")?;

        Ok(DynBounds {
            binder,
            traits,
        })
    }

    fn parse_dyn_trait(&mut self) -> Result<DynTrait, String> {
        let path = self.parse_path()?;

        let mut assoc_type_bindings = Vec::new();
        while self.cur() == b'p' {
            assoc_type_bindings.push(self.parse_dyn_trait_assoc_binding()?);
        }

        Ok(DynTrait {
            path,
            assoc_type_bindings,
        })
    }

    fn parse_dyn_trait_assoc_binding(&mut self) -> Result<DynTraitAssocBinding, String> {
        self.eat(b'p', "<dyn-trait-assoc-binding>")?;
        Ok(DynTraitAssocBinding {
            ident: self.parse_uident()?,
            ty: self.parse_type()?,
        })
    }

    fn parse_type(&mut self) -> Result<Type, String> {
        let tag = self.cur();
        self.pos += 1;

        Ok(match tag {
            b'a' => Type::BasicType(BasicType::I8),
            b'b' => Type::BasicType(BasicType::Bool),
            b'c' => Type::BasicType(BasicType::Char),
            b'd' => Type::BasicType(BasicType::F64),
            b'e' => Type::BasicType(BasicType::Str),
            b'f' => Type::BasicType(BasicType::F32),
            b'h' => Type::BasicType(BasicType::U8),
            b'i' => Type::BasicType(BasicType::Isize),
            b'j' => Type::BasicType(BasicType::Usize),
            b'l' => Type::BasicType(BasicType::I32),
            b'm' => Type::BasicType(BasicType::U32),
            b'n' => Type::BasicType(BasicType::I128),
            b'o' => Type::BasicType(BasicType::U128),
            b'p' => Type::BasicType(BasicType::Placeholder),
            b's' => Type::BasicType(BasicType::I16),
            b't' => Type::BasicType(BasicType::U16),
            b'u' => Type::BasicType(BasicType::Unit),
            b'v' => Type::BasicType(BasicType::Ellipsis),
            b'x' => Type::BasicType(BasicType::I64),
            b'y' => Type::BasicType(BasicType::U64),
            b'z' => Type::BasicType(BasicType::Never),

            b'A' => {
                Type::Array(Arc::new(self.parse_type()?), Arc::new(self.parse_const()?))
            }

            b'S' => {
                Type::Slice(Arc::new(self.parse_type()?))
            }

            b'C' | b'M' | b'X' | b'Y' |b'N' | b'I' => {
                self.pos -= 1;
                Type::Named(Arc::new(self.parse_path()?))
            }

            b'T' => {
                let mut args = Vec::new();
                while self.cur() != b'E' {
                    args.push(self.parse_type()?);
                }

                self.eat(b'E', "<type>")?;

                Type::Tuple(args)
            }

            b'R' => {
                let lifetime = if self.cur() == b'L' {
                    Some(self.parse_lifetime()?)
                } else {
                    None
                };

                Type::Ref(lifetime, Arc::new(self.parse_type()?))
            }

            b'Q' => {
                let lifetime = if self.cur() == b'L' {
                    Some(self.parse_lifetime()?)
                } else {
                    None
                };

                Type::RefMut(lifetime, Arc::new(self.parse_type()?))
            }

            b'P' => {
                Type::RawPtrConst(Arc::new(self.parse_type()?))
            }

            b'O' => {
                Type::RawPtrMut(Arc::new(self.parse_type()?))
            }

            b'F' => {
                Type::Fn(Arc::new(self.parse_fn_sig()?))
            }

            b'D' => {
                Type::DynTrait(Arc::new(self.parse_dyn_bounds()?), self.parse_lifetime()?)
            }

            b'B' => {
                let mut parser = self.parse_backref()?;
                parser.parse_type()?
            }

            c => {
                return Err(format!("Expected start of <type>, found {} instead.", c as char));
            }
        })
    }

    fn parse_impl_path(&mut self) -> Result<ImplPath, String> {
        let dis = if self.cur() == b's' {
            Some(self.parse_disambiguator()?)
        } else {
            None
        };

        Ok(ImplPath {
            dis,
            path: Arc::new(self.parse_path()?),
        })
    }

    fn parse_path(&mut self) -> Result<Path, String> {
        let tag = self.cur();
        self.pos += 1;

        Ok(match tag {
            b'C' => {
                Path::CrateRoot {
                    id: self.parse_ident()?,
                }
            }
            b'M' => {
                Path::InherentImpl {
                    impl_path: self.parse_impl_path()?,
                    self_type: self.parse_type()?,
                }
            }
            b'X' => {
                Path::TraitImpl {
                    impl_path: self.parse_impl_path()?,
                    self_type: self.parse_type()?,
                    trait_name: Arc::new(self.parse_path()?),
                }
            }
            b'Y' => {
                Path::TraitDef {
                    self_type: self.parse_type()?,
                    trait_name: Arc::new(self.parse_path()?),
                }
            }
            b'N' => {
                Path::Nested {
                    ns: self.parse_namespace()?,
                    inner: Arc::new(self.parse_path()?),
                    ident: self.parse_ident()?,
                }
            }
            b'I' => {
                let inner = self.parse_path()?;

                let mut args = Vec::new();
                while self.cur() != b'E' {
                    args.push(self.parse_generic_arg()?);
                }

                self.eat(b'E', "<path>")?;

                Path::Generic {
                    inner: Arc::new(inner),
                    args,
                }
            }
            b'B' => {
                let mut parser = self.parse_backref()?;
                parser.parse_path()?
            }
            other => {
                return expected("CMXYNIB", other, "parsing", "<path>");
            }
        })
    }

    fn parse_namespace(&mut self) -> Result<Namespace, String> {
        let c = self.cur();

        match c {
            b'A' ..= b'Z' | b'a' ..= b'z' => {}
            c => return Err(format!("Invalid namespace character '{}'", c))
        };

        self.pos += 1;

        Ok(Namespace(c))
    }

    fn parse_ident(&mut self) -> Result<Ident, String> {
        let dis = if self.cur() == b's' {
            self.parse_disambiguator()?
        } else {
            Base62Number(0)
        };

        Ok(Ident {
            dis,
            u_ident: self.parse_uident()?
        })
    }

    fn parse_disambiguator(&mut self) -> Result<Base62Number, String> {
        self.eat(b's', "<disambiguator>")?;

        Ok(Base62Number(self.parse_base62_number()?.0 + 1))
    }

    fn parse_uident(&mut self) -> Result<UIdent, String> {
        let punycode = self.try_eat(b'u');
        let DecimalNumber(num_bytes) = self.parse_decimal_number()?;
        let start = self.pos;
        let end = start + num_bytes as usize;

        if end > self.input.len() {
            return Err(format!("identifier extend beyond end of input"));
        }

        self.pos = end;

        let bytes = &self.input[start.. end];

        let ident = if punycode {
            charset::decode_punycode_ident(bytes)?
        } else {
            String::from_utf8(bytes.to_owned()).map_err(|e| {
                format!("{:?}", e)
            })?
        };

        Ok(UIdent(ident))
    }


    fn parse_decimal_number(&mut self) -> Result<DecimalNumber, String> {
        Ok(DecimalNumber(self.parse_number(10)?))
    }

    fn parse_base62_number(&mut self) -> Result<Base62Number, String> {

        let n = if self.cur() == b'_' {
            0
        } else {
            self.parse_number(62)? + 1
        };

        self.eat(b'_', "<base-62-number>")?;

        Ok(Base62Number(n))
    }

    fn cur(&self) -> u8 {
        if self.pos < self.input.len() {
            self.input[self.pos]
        } else {
            EOT
        }
    }

    #[must_use]
    fn eat(&mut self, c: u8, noun: &str) -> Result<(), String> {
        if self.cur() != c {
            return expected(str::from_utf8(&[c]).unwrap(), self.cur(), "parsing", noun);
        }

        self.pos += 1;

        Ok(())
    }

    fn try_eat(&mut self, c: u8) -> bool {
        if self.cur() == c {
            self.pos += 1;
            true
        } else {
            false
        }
    }

    #[must_use]
    fn parse_number(&mut self, radix: u8) -> Result<u64, String> {
        if ascii_digit_to_value(self.cur(), radix).is_none() {
            return Err(format!(
                "expected base-{} digit, found {:?}",
                radix,
                self.cur() as char
            ));
        }

        let mut value = 0;

        while let Some(digit) = ascii_digit_to_value(self.cur(), radix) {
            value = value * (radix as u64) + digit;
            self.pos += 1;
        }

        Ok(value)
    }

    #[must_use]
    fn parse_backref(&mut self) -> Result<Parser<'input>, String> {
        let Base62Number(pos) = self.parse_base62_number()?;

        // Account for the `_R` prefix
        let pos = pos + 2;

        Ok(Parser {
            input: self.input,
            pos: pos as usize,
        })
    }
}
