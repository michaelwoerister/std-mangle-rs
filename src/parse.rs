use ast::*;
use error::expected;
use std::str;
use std::sync::Arc;

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

        let path = parser.parse_qname().map_err(|s| {
            format!(
                "In {:?}: Parsing error at pos {}: {}",
                str::from_utf8(mangled).unwrap(),
                parser.pos,
                s
            )
        })?;

        Ok(Symbol {
            name: path,
            // instantiating_crate: panic!(),
        })
    }

    fn cur(&self) -> u8 {
        self.input[self.pos]
    }

    fn cur_char(&self) -> char {
        self.cur() as char
    }

    fn parse_symbol_prefix(&mut self) -> Result<(), String> {
        assert_eq!(self.pos, 0);
        if &self.input[0..2] != b"_R" {
            return Err("Not a Rust symbol".to_owned());
        }

        self.pos += 2;

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
        let to_digit = |byte| {
            let value = match byte {
                b'0'..=b'9' => byte - b'0',
                b'a'..=b'z' => (byte - b'a') + 10,
                b'A'..=b'Z' => (byte - b'A') + 10,
                _ => return None,
            };

            if value < radix {
                Some(value)
            } else {
                None
            }
        };

        if to_digit(self.cur()).is_none() {
            return Err(format!(
                "expected base-{} digit, found {:?}",
                radix,
                self.cur_char()
            ));
        }

        let mut value = 0;

        while let Some(digit) = to_digit(self.cur()) {
            value = value * (radix as u64) + digit as u64;
            self.pos += 1;
        }

        Ok(value)
    }

    fn parse_len_prefixed_ident(&mut self) -> Result<Ident, String> {
        let len = self.parse_number(10)? as usize;
        let ident = String::from_utf8(self.input[self.pos..self.pos + len].to_owned()).unwrap();
        self.pos += len;

        let tag = match self.cur() {
            b'F' => {
                self.pos += 1;
                IdentTag::Function
            }
            b'S' => {
                self.pos += 1;
                IdentTag::Static
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

    fn parse_qname(&mut self) -> Result<Arc<QName>, String> {
        match self.cur() {
            b'N' => {
                self.pos += 1;

                let name = self.parse_name_prefix()?;

                let args = if self.cur() == b'I' {
                    self.parse_generic_argument_list()?
                } else {
                    GenericArgumentList::new_empty()
                };

                if self.cur() != b'E' {
                    return expected("E", self.cur(), "parsing", "<qualified-name>");
                }

                self.pos += 1;

                Ok(Arc::new(QName::Name { name, args }))
            }
            b'S' => {
                let subst = self.parse_subst()?;
                Ok(Arc::new(QName::Subst(subst)))
            }
            c => {
                return expected("NS", c, "parsing", "<qualified-name>");
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

    fn parse_name_prefix(&mut self) -> Result<Arc<NamePrefix>, String> {
        let root = Arc::new(match self.cur() {
            b'S' => NamePrefix::Subst(self.parse_subst()?),

            b'X' => {
                self.pos += 1;

                let self_type = self.parse_type()?;
                let impled_trait = self.parse_qname()?;
                let dis = self.parse_opt_numeric_disambiguator()?;

                NamePrefix::TraitImpl {
                    self_type,
                    impled_trait,
                    dis,
                }
            }

            b'M' => {
                self.pos += 1;

                let self_type = self.parse_type()?;

                NamePrefix::InherentImpl { self_type }
            }

            c if c.is_ascii_digit() => {
                let ident = self.parse_len_prefixed_ident()?;

                if let Some(sep) = ident.ident.rfind('_') {
                    NamePrefix::CrateId {
                        name: ident.ident[..sep].to_owned(),
                        dis: ident.ident[sep + 1..].to_owned(),
                    }
                } else {
                    return Err(format!("Crate ID does not contain disambiguator"));
                }
            }

            c => {
                return expected("SX#", c, "parsing", "<name-prefix>");
            }
        });

        let mut path = root;

        while self.cur() != b'E' && self.cur() != b'I' {
            let ident = self.parse_len_prefixed_ident()?;

            path = Arc::new(NamePrefix::Node {
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
                Type::Named(self.parse_qname()?)
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
            _ => {
                return Err(format!("Unknown ABI spec {:?}", self.cur_char()));
            }
        };

        self.pos += 1;

        Ok(abi)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    quickcheck! {
        fn parsing(symbol: Symbol) -> bool {
            let mut mangled = String::new();
            symbol.mangle(&mut mangled);
            match Parser::parse(mangled.as_bytes()) {
                Ok(parsed) => {
                    if symbol != parsed {
                        panic!("expected: {:?}\n\
                                actual:   {:?}\n\
                                mangled:  {}\n",
                                symbol,
                                parsed,
                                mangled)
                    } else {
                        true
                    }
                }
                Err(e) => {
                    panic!("{}", e)
                }
            }
        }
    }

    quickcheck! {
        fn parsing_compressed(symbol: Symbol) -> bool {
            let mut mangled = String::new();
            let compressed = ::compress::compress(&symbol);
            compressed.mangle(&mut mangled);
            match Parser::parse(mangled.as_bytes()) {
                Ok(parsed) => {
                    if parsed != compressed {
                        panic!("Re-parsed compressed symbol differs from original
                                expected: {:?}\n\
                                actual:   {:?}\n\
                                mangled:  {}\n",
                                compressed,
                                parsed,
                                mangled)
                    } else {
                        true
                    }
                }
                Err(e) => {
                    panic!("{}", e)
                }
            }
        }
    }
}
