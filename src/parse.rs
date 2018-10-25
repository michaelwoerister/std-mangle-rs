
use ast::*;

struct Parser<'in> {
    in: &'in [u8],
    pos: usize,
}

impl<'input> Parser<'input> {

    pub fn parse(mangled: &[u8]) -> Result<Symbol, String> {

        let mut parser = Parser {
            in: mangled,
            pos: 0,
        };

        self.parse_symbol_prefix()?;

        let qname = self.parse_fully_qualified_name()?;

        panic!();
    }

    fn cur(&self) -> u8 {
        self.in[self.pos]
    }

    fn cur_char(&self) -> char {
        self.cur() as char
    }

    fn parse_symbol_prefix(&mut self) -> Result<(), String> {
        assert_eq!(self.pos, 0);
        if &self.in[0 .. 2] != b"_R" {
            return Err("Not a Rust symbol");
        }

        self.pos += 2;

        Ok(())
    }

    fn parse_len_prefixed_ident(&mut self) -> Result<Ident, String> {
        let len = self.parse_decimal()?;
        let ident = String::from_utf8(self.in[self.pos .. self.pos + len].to_owned())
                           .unwrap();
        self.pos += len;

        let tag = match self.cur() {
            b'V' => {
                self.pos += 1;
                IdentTag::ValueNs
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
            let dis = self.parse_decimal()?;

            if self.cur() != '_' {
                return Err(format!("expected '_', found '{}'", self.cur() as char));
            }

            self.pos += 1;
        } else {
            0
        };

        Ok(Ident {
            ident,
            tag,
            dis,
        })
    }

    fn parse_decimal(&mut self) -> Result<usize, String> {
        if !self.cur().is_ascii_digit() {
            return Err(format!("expected digit, found {}", self.cur()));
        }

        let mut value = self.cur() - b'0';
        self.pos += 1;
        while self.cur().is_ascii_digit() {
            let digit = self.cur() - b'0';
            value = value * 10 + digit;
            self.pos += 1;
        }

        Ok(value)
    }

    fn parse_fully_qualified_name(&mut self) -> Result<Arc<FullyQualifiedName>, String> {

        // TODO subst

        let name = self.parse_name_prefix_with_params()?;
        assert_eq!(self.in[self.pos], b'E');
        self.pos += 1;

        Ok(Arc::new(FullyQualifiedName::Name {
            name
        }))
    }

    fn parse_subst(&mut self) -> Result<Subst, String> {
        if self.cur() != 'S' {
            return Err(format!("expected 'S', found {:?}", self.cur_char()));
        }

        self.pos += 1;

        let index = if self.cur() == b'_' {
            0
        } else {
            let n = self.parse_decimal()?;
            n + 1
        };

        if self.cur() != '_' {
            return Err(format!("expected '_', found {:?}", self.cur_char()));
        }

        self.pos += 1;

        Ok(Subst(index))
    }

    fn parse_name_prefix_with_params(&mut self) -> Result<Arc<NamePrefixWithParams>, String> {

        let root = Arc::new(match self.in[self.pos] {
            b'S' => {
                NamePrefix::Subst(self.parse_subst())
            }
            b'X' => {
                //
                self.pos += 1;

                let self_type = self.parse_type()?;
                let impled_trait = self.parse_fully_qualified_name()?;

                NamePrefix::TraitImpl {

                }
            }
            c if c.is_ascii_digit() => {

                let ident = self.parse_len_prefixed_ident()?;

                // TODO: split
                // parse ident
                // let crate_id = self.parse_crate_id()?;

                NamePrefix::CrateId {
                    name: String::new(),
                    dis: String::new(),
                }
            }

            other => {
                return Err(format!("expected 'S', 'X', or digit, found '{}'");)
            }
        });

        let root = if self.in[self.pos] == b'S' {
            let subst = self.parse_subst()?;

            let params = if self.in[self.pos] == b'I' {
                self.parse_generic_argument_list()?
            } else {
                vec![]
            };

            Arc::new(NamePrefixWithParams {
                prefix: NamePrefix
            })
        } else {

        }

        while self.in[self.pos] != b'E' {

        }
    }
}


