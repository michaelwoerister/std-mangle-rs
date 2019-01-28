use ast::*;
use charset;
use error::{self, expected};
use int_radix::ascii_digit_to_value;
use std::str;
use std::sync::Arc;

pub const EOT: u8 = 5; // ASCII "end of transmission"

pub struct Parser<'input> {
    input: &'input [u8],
    pos: usize,
}

impl<'input> Parser<'input> {
    pub fn parse(mangled: &[u8]) -> Result<Symbol, String> {
        let mut parser = Parser {
            input: mangled,
            pos: 0,
        };

        parser.parse_symbol_prefix()?;

        let path = parser.parse_abs_path().map_err(|s| {
            format!(
                "In {:?}: Parsing error at pos {}: {}",
                str::from_utf8(mangled).unwrap(),
                parser.pos,
                s
            )
        })?;

        let instantiating_crate = if parser.pos < parser.input.len() {
            Some(parser.parse_path_prefix()?)
        } else {
            None
        };

        Ok(Symbol {
            name: path,
            instantiating_crate,
        })
    }

    fn cur(&self) -> u8 {
        if self.pos < self.input.len() {
            self.input[self.pos]
        } else {
            EOT
        }
    }

    fn parse_symbol_prefix(&mut self) -> Result<(), String> {
        assert_eq!(self.pos, 0);

        if &self.input[0..2] != b"_R" {
            return Err("Not a Rust symbol".to_owned());
        }

        self.pos += 2;

        if self.cur().is_ascii_digit() {
            let encoding_version = self.parse_number(10)? + 1;
            return error::version_mismatch(encoding_version, 0);
        }

        Ok(())
    }

    fn parse_opt_numeric_disambiguator(&mut self) -> Result<NumericDisambiguator, String> {
        let index = if self.cur() != b's' {
            0
        } else {
            self.pos += 1;

            if self.cur() == b'_' {
                self.pos += 1;
                1
            } else {
                let n = self.parse_underscore_terminated_number(NUMERIC_DISAMBIGUATOR_RADIX)?;
                n + 2
            }
        };

        Ok(NumericDisambiguator(index))
    }

    fn parse_underscore_terminated_number(&mut self, radix: u8) -> Result<u64, String> {
        let value = self.parse_number(radix)?;

        if self.cur() != b'_' {
            return expected(
                "_",
                self.cur(),
                "parsing",
                "<underscored-terminated number>",
            );
        }

        self.pos += 1;
        Ok(value)
    }

    fn parse_number(&mut self, radix: u8) -> Result<u64, String> {
        if ascii_digit_to_value(self.cur(), radix).is_none() {
            return Err(format!(
                "Expected base-{} digit; found '{}' instead;",
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

    fn parse_len_prefixed_ident(&mut self) -> Result<Ident, String> {
        let len = self.parse_number(10)? as usize;
        let ident_bytes = &self.input[self.pos..self.pos + len];

        if ident_bytes.iter().any(|b| !b.is_ascii()) {
            return Err(format!(
                "Ident '{}' unexpectedly contains non-ascii characters.",
                String::from_utf8_lossy(ident_bytes)
            ));
        }

        self.pos += len;

        let ident = if self.cur() == b'u' {
            self.pos += 1;
            match charset::decode_punycode_ident(ident_bytes) {
                Ok(ident) => ident,
                Err(err) => return Err(err),
            }
        } else {
            String::from_utf8(ident_bytes.to_owned()).unwrap()
        };

        let tag = match self.cur() {
            b'V' => {
                self.pos += 1;
                IdentTag::ValueNs
            }
            b'C' => {
                self.pos += 1;
                IdentTag::Closure
            }
            _ => IdentTag::TypeNs,
        };

        let dis = self.parse_opt_numeric_disambiguator()?;

        Ok(Ident { ident, tag, dis })
    }

    fn parse_abs_path(&mut self) -> Result<Arc<AbsolutePath>, String> {
        match self.cur() {
            b'N' => {
                self.pos += 1;

                let name = self.parse_path_prefix()?;

                let args = if self.cur() == b'I' {
                    self.parse_generic_argument_list()?
                } else {
                    GenericArgumentList::new_empty()
                };

                if self.cur() != b'E' {
                    return expected("E", self.cur(), "parsing", "<absolute-path>");
                }

                self.pos += 1;

                Ok(Arc::new(AbsolutePath::Path { name, args }))
            }
            b'S' => {
                let subst = self.parse_subst()?;
                Ok(Arc::new(AbsolutePath::Subst(subst)))
            }
            c => {
                return expected("NS", c, "parsing", "<absolute-path>");
            }
        }
    }

    fn parse_subst(&mut self) -> Result<Subst, String> {
        if self.cur() != b'S' {
            return expected("S", self.cur(), "parsing", "<substitution>");
        }

        self.pos += 1;

        let index = if self.cur() == b'_' {
            self.pos += 1;
            0
        } else {
            let n = self.parse_underscore_terminated_number(SUBST_RADIX)?;
            n + 1
        };

        Ok(Subst(index))
    }

    fn parse_path_prefix(&mut self) -> Result<Arc<PathPrefix>, String> {
        let root = Arc::new(match self.cur() {
            b'S' => PathPrefix::Subst(self.parse_subst()?),

            b'X' => {
                self.pos += 1;

                let self_type = self.parse_type()?;

                let impled_trait = if self.cur() == b'N' || self.cur() == b'S' {
                    Some(self.parse_abs_path()?)
                } else {
                    None
                };
                let dis = self.parse_opt_numeric_disambiguator()?;

                PathPrefix::TraitImpl {
                    self_type,
                    impled_trait,
                    dis,
                }
            }

            b'N' => {
                PathPrefix::AbsolutePath {
                    path: self.parse_abs_path()?,
                }
            }

            c if c.is_ascii_digit() => {
                let ident = self.parse_len_prefixed_ident()?;

                if let Some(sep) = ident.ident.rfind('_') {
                    PathPrefix::CrateId {
                        name: ident.ident[..sep].to_owned(),
                        dis: ident.ident[sep + 1..].to_owned(),
                    }
                } else {
                    return Err(format!("Crate ID does not contain disambiguator"));
                }
            }

            c => {
                return expected("SXN#", c, "parsing", "<name-prefix>");
            }
        });

        let mut path = root;

        while self.cur() != EOT && self.cur() != b'E' && self.cur() != b'I' {
            let ident = self.parse_len_prefixed_ident()?;

            path = Arc::new(PathPrefix::Node {
                prefix: path,
                ident,
            });
        }

        Ok(path)
    }

    fn parse_generic_argument_list(&mut self) -> Result<GenericArgumentList, String> {
        assert_eq!(self.cur(), b'I');

        self.pos += 1;

        let mut args = vec![];

        while self.cur() != b'E' {
            args.push(self.parse_type()?);
        }

        self.pos += 1;

        Ok(GenericArgumentList(args))
    }

    fn parse_type(&mut self) -> Result<Arc<Type>, String> {
        let tag = self.cur();
        self.pos += 1;
        let t = match tag {
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
            b's' => Type::BasicType(BasicType::I16),
            b't' => Type::BasicType(BasicType::U16),
            b'u' => Type::BasicType(BasicType::Unit),
            b'v' => Type::BasicType(BasicType::Ellipsis),
            b'x' => Type::BasicType(BasicType::I64),
            b'y' => Type::BasicType(BasicType::U64),
            b'z' => Type::BasicType(BasicType::Never),

            b'A' => {
                let len = if self.cur().is_ascii_digit() {
                    Some(self.parse_number(10)?)
                } else {
                    None
                };

                let inner = self.parse_type()?;

                Type::Array(len, inner)
            }

            b'F' => {
                let is_unsafe = if self.cur() == b'U' {
                    self.pos += 1;
                    true
                } else {
                    false
                };

                let abi = if self.cur() == b'K' {
                    self.parse_abi()?
                } else {
                    Abi::Rust
                };

                let mut params = vec![];
                while self.cur() != b'E' && self.cur() != b'J' {
                    params.push(self.parse_type()?);
                }

                let return_type = if self.cur() == b'J' {
                    self.pos += 1;
                    Some(self.parse_type()?)
                } else {
                    None
                };

                if self.cur() != b'E' {
                    return expected("E", self.cur(), "parsing", "<fn-type>");
                }

                // Skip the 'E'
                self.pos += 1;

                Type::Fn {
                    is_unsafe,
                    abi,
                    return_type,
                    params,
                }
            }

            b'G' => {
                let ident = self.parse_len_prefixed_ident()?;
                if self.cur() != b'E' {
                    return expected("E", self.cur(), "parsing", "<generic-param-name>");
                }
                self.pos += 1;
                Type::GenericParam(ident)
            }

            b'N' => {
                // We have to back up here
                self.pos -= 1;
                Type::Named(self.parse_abs_path()?)
            }

            b'O' => Type::RawPtrMut(self.parse_type()?),
            b'P' => Type::RawPtrConst(self.parse_type()?),
            b'Q' => Type::RefMut(self.parse_type()?),
            b'R' => Type::Ref(self.parse_type()?),
            b'S' => {
                self.pos -= 1;
                Type::Subst(self.parse_subst()?)
            }
            b'T' => {
                let mut params = vec![];

                while self.cur() != b'E' {
                    params.push(self.parse_type()?);
                }

                self.pos += 1;

                Type::Tuple(params)
            }

            other => {
                return Err(format!(
                    "expected start of type; found '{}' instead; \
                     while parsing <type>",
                    other as char
                ));
            }
        };

        Ok(Arc::new(t))
    }

    fn parse_abi(&mut self) -> Result<Abi, String> {
        if self.cur() != b'K' {
            return expected("K", self.cur(), "parsing", "<abi>");
        }

        self.pos += 1;

        let abi = match self.cur() {
            b'c' => Abi::C,
            c => {
                return expected("c", c, "parsing", "<abi>");
            }
        };

        self.pos += 1;

        Ok(abi)
    }
}
