
use ast::*;
use std::sync::Arc;
use std::str;

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

        let path = parser.parse_fully_qualified_name()
                         .map_err(|s| format!("In {:?}: Parsing error at pos {}: {}",
                            str::from_utf8(mangled).unwrap(),
                            parser.pos,
                            s))?;

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
        if &self.input[0 .. 2] != b"_R" {
            return Err("Not a Rust symbol".to_owned());
        }

        self.pos += 2;

        Ok(())
    }

    fn parse_len_prefixed_ident(&mut self) -> Result<Ident, String> {
        let len = self.parse_decimal()?;
        let ident = String::from_utf8(self.input[self.pos .. self.pos + len].to_owned())
                           .unwrap();
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
            _ => {
                IdentTag::TypeNs
            }
        };

        let dis = if self.cur() == b's' {
            self.pos += 1;

            let dis = if self.cur() == b'_' {
                1
            } else {
                self.parse_decimal()? + 2
            };

            if self.cur() != b'_' {
                return Err(format!("expected '_', found '{}'", self.cur() as char));
            }

            self.pos += 1;

            dis
        } else {
            0
        };

        Ok(Ident {
            ident,
            tag,
            dis: dis as u32,
        })
    }

    fn parse_decimal(&mut self) -> Result<usize, String> {
        if !self.cur().is_ascii_digit() {
            return Err(format!("expected digit, found {:?}", self.cur_char()));
        }

        let mut value = (self.cur() - b'0') as usize;
        self.pos += 1;
        while self.cur().is_ascii_digit() {
            let digit = self.cur() - b'0';
            value = value * 10 + digit as usize;
            self.pos += 1;
        }

        Ok(value)
    }

    fn parse_fully_qualified_name(&mut self) -> Result<Arc<FullyQualifiedName>, String> {

        match self.cur() {
            b'N' => {
                self.pos += 1;

                let name = self.parse_name_prefix_with_params()?;
                assert_eq!(self.cur(), b'E');
                self.pos += 1;

                Ok(Arc::new(FullyQualifiedName::Name {
                    name
                }))
            }
            b'S' => {
                let subst = self.parse_subst()?;
                Ok(Arc::new(FullyQualifiedName::Subst(subst)))
            }
            _ => {
                return Err(format!("Expected 'N' or 'S', found {:?}", self.cur_char()));
            }
        }
    }

    fn parse_subst(&mut self) -> Result<Subst, String> {
        if self.cur() != b'S' {
            return Err(format!("expected 'S', found {:?}", self.cur_char()));
        }

        self.pos += 1;

        let index = if self.cur() == b'_' {
            0
        } else {
            let n = self.parse_decimal()?;
            n + 1
        };

        if self.cur() != b'_' {
            return Err(format!("expected '_', found {:?}", self.cur_char()));
        }

        self.pos += 1;

        Ok(Subst(index))
    }

    fn parse_name_prefix_with_params(&mut self) -> Result<Arc<NamePrefixWithParams>, String> {

        let root = Arc::new(match self.cur() {
            b'S' => {
                NamePrefix::Subst(self.parse_subst()?)
            }

            b'X' => {
                self.pos += 1;

                let self_type = self.parse_type()?;
                let impled_trait = self.parse_fully_qualified_name()?;

                NamePrefix::TraitImpl {
                    self_type,
                    impled_trait,
                }
            }

            c if c.is_ascii_digit() => {

                let ident = self.parse_len_prefixed_ident()?;

                if let Some(sep) = ident.ident.rfind('_') {
                    NamePrefix::CrateId {
                        name: ident.ident[..sep].to_owned(),
                        dis: ident.ident[sep + 1 ..].to_owned(),
                    }
                } else {
                    return Err(format!("Crate ID does not contain disambiguator"));
                }
            }

            _ => {
                return Err(format!("expected 'S', 'X', or digit, found {:?}", self.cur_char()));
            }
        });

        let mut path = match *root {
            NamePrefix::Subst(subst) => {
                if self.cur() == b'I' {
                    Arc::new(NamePrefixWithParams::Node {
                        prefix: root.clone(),
                        args: self.parse_generic_argument_list()?,
                    })
                } else {
                    // The substitution always signifies the whole
                    // NamePrefixWithParams production in this case
                    Arc::new(NamePrefixWithParams::Subst(subst))
                }
            }
            _ => {
                if self.cur() == b'I' {
                    return Err("Did not expect path root to have parameter list".to_owned());
                }

                Arc::new(NamePrefixWithParams::Node {
                    prefix: root.clone(),
                    args: GenericArgumentList::new_empty(),
                })
            }
        };

        while self.cur() != b'E' {
            let ident = self.parse_len_prefixed_ident()?;

            let prefix = Arc::new(NamePrefix::Node {
                prefix: path,
                ident,
            });

            let args = if self.cur() == b'I' {
                self.parse_generic_argument_list()?
            } else {
                GenericArgumentList::new_empty()
            };

            path = Arc::new(NamePrefixWithParams::Node {
                prefix,
                args,
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

        Ok(GenericArgumentList {
            params: args,
        })
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
            b'v' => Type::BasicType(BasicType::Unit),
            b'x' => Type::BasicType(BasicType::I64),
            b'y' => Type::BasicType(BasicType::U64),
            b'z' => Type::BasicType(BasicType::Never),

            b'A' => {
                let len = if self.cur().is_ascii_digit() {
                    Some(self.parse_decimal()?)
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

                let is_variadic = if self.cur() == b'L' {
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

                let return_type = self.parse_type()?;

                let mut params = vec![];
                while self.cur() != b'E' {
                    params.push(self.parse_type()?);
                }

                // Skip the 'E'
                self.pos += 1;

                Type::Fn {
                    is_unsafe,
                    is_variadic,
                    abi,
                    return_type,
                    params,
                }
            }

            b'G' => {
                let ident = self.parse_len_prefixed_ident()?;
                if self.cur() != b'E' {
                    return Err(format!("While parsing generic parameter name: Expected 'E', found {:?}", self.cur_char()));
                }
                self.pos += 1;
                Type::GenericParam(ident.ident)
            }

            b'N' => {
                // We have to back up here
                self.pos -= 1;
                Type::Named(self.parse_fully_qualified_name()?)
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
                return Err(format!("expected start of type, found {:?}", other as char));
            }
        };

        Ok(Arc::new(t))
    }

    fn parse_abi(&mut self) -> Result<Abi, String> {
        if self.cur() != b'K' {
            return Err(format!("Expected start of <abi> ('K'), found {:?}", self.cur_char()));
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
