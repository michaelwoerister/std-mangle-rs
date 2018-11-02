use std::io::Write;
use std::str;

pub struct Demangler<'input> {
    pos: usize,

    input: &'input [u8],
    out: Vec<u8>,
    dict: Vec<(usize, usize)>,
}

impl<'input> Demangler<'input> {

    pub fn demangle(input: &[u8]) -> Result<String, String> {
        let mut state = Demangler {
            pos: 0,
            input,
            out: Vec::new(),
            dict: Vec::new(),
        };

        if let Err(msg) = state.demangle_symbol() {
            return Err(format!("Error demangling at position {}: {} - {}",
                               state.pos,
                               msg,
                               str::from_utf8(input).unwrap()));
        }

        String::from_utf8(state.out).map_err(|e| format!("{}", e))
    }

    fn demangle_symbol(&mut self) -> Result<(), String> {
        assert!(self.pos == 0);
        if &self.input[0..2] != b"_R" {
            return Err("Not a Rust symbol".to_string())
        }

        self.pos = 2;
        self.demangle_fully_qualified_name()
    }

    fn cur(&self) -> u8 {
        self.input[self.pos]
    }

    fn alloc_subst(&mut self, start: usize) {
        let end = self.out.len();
        self.dict.push((start, end));
    }

    fn parse_decimal(&mut self) -> Result<usize, String> {
        if !self.cur().is_ascii_digit() {
            return Err(format!("expected digit, found {:?}", self.cur() as char));
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

    fn demangle_fully_qualified_name(&mut self) -> Result<(), String> {

        match self.cur() {
            b'N' => {
                let subst_start = self.out.len();
                self.pos += 1;

                self.demangle_name_prefix()?;

                if self.cur() == b'I' {
                    self.demangle_generic_argument_list()?;
                    self.alloc_subst(subst_start);
                } else {
                    // Don't add a subst because the name prefix already contains the
                    // whole output generated for this name.
                }

                assert_eq!(self.cur(), b'E');
                self.pos += 1;
            }

            b'S' => {
                self.demangle_subst()?;
            }

            c => {
                return Err(format!("Expected 'S' or 'N', found '{}'", c as char));
            }
        }

        Ok(())
    }

    fn demangle_ident(&mut self) -> Result<(), String> {
        if !self.cur().is_ascii_digit() {
            return Err(format!("idents must start with length-spec; found '{}'", self.cur() as char));
        }

        let ident_len = self.parse_decimal()?;
        let ident_start = self.pos;

        self.pos += ident_len;

        let mut always_print_dis = false;

        match self.cur() {
            b'C' => {
                self.out.extend_from_slice(b"{closure}");
                self.pos += 1;
                always_print_dis = true;
            }
            b'F' | b'S' => {
                self.out.extend_from_slice(&self.input[ident_start .. ident_start + ident_len]);
                // Just skip
                self.pos += 1;
            }
            _ => {
                self.out.extend_from_slice(&self.input[ident_start .. ident_start + ident_len]);
            }
        }

        let index = if self.cur() == b's' {
            self.pos += 1;
            let index = if self.cur() == b'_' {
                2
            } else {
                self.parse_decimal()? + 3
            };
            assert_eq!(self.cur(), b'_');
            self.pos += 1;
            index
        } else {
            1
        };

        if index > 1 || always_print_dis {
            write!(self.out, "'{}", index).unwrap();
        }

        Ok(())
    }

    fn demangle_name_prefix(&mut self) -> Result<(), String> {

        let subst_start = self.out.len();

        // parse the root
        let add_root_to_dict = match self.cur() {
            b'S' => {
                self.demangle_subst()?;
                false
            }

            b'X' => {
                self.pos += 1;

                self.out.push(b'<');
                self.demangle_type()?;
                self.out.extend_from_slice(b" as ");
                self.demangle_fully_qualified_name()?;
                self.out.push(b'>');
                true
            }

            b'M' => {
                self.pos += 1;
                self.demangle_type()?;
                // The type already added a subst
                false
            }

            c if c.is_ascii_digit() => {

                let len = self.parse_decimal()?;
                let name_and_dis = &self.input[self.pos .. self.pos + len];

                if let Some(sep) = name_and_dis.iter().rposition(|&c| c == b'_') {
                    self.out.extend_from_slice(&name_and_dis[..sep]);
                    self.out.push(b'[');
                    self.out.extend_from_slice(&name_and_dis[sep + 1 ..]);
                    self.out.push(b']');
                } else {
                    return Err(format!("Crate ID '{}' does not contain disambiguator",
                                        str::from_utf8(name_and_dis).unwrap()));
                };

                self.pos += len;

                true
            }

            c => {
                return Err(format!("expected 'S', 'X', or digit, found {:?}", c as char));
            }
        };

        if add_root_to_dict {
            self.alloc_subst(subst_start);
        }

        while self.cur() != b'E' && self.cur() != b'I' {
            self.out.extend_from_slice(b"::");
            self.demangle_ident()?;
            self.alloc_subst(subst_start);
        }

        Ok(())
    }

    fn demangle_generic_argument_list(&mut self) -> Result<(), String> {
        assert_eq!(self.cur(), b'I');

        self.pos += 1;
        self.out.push(b'<');

        while self.cur() != b'E' {
            self.demangle_type()?;
            self.out.push(b',');
        }

        assert_eq!(self.cur(), b'E');
        self.pos += 1;

        assert_eq!(self.out.last(), Some(&b','));
        *self.out.last_mut().unwrap() = b'>';

        Ok(())
    }

    fn demangle_type(&mut self) -> Result<(), String> {

        let tag = self.cur();
        self.pos += 1;

        let subst_start = self.out.len();

        let push_basic_type = |this: &mut Self, s: &[u8]| {
            this.out.extend_from_slice(s);
            Ok(())
        };

        match tag {
            b'a' => return push_basic_type(self, b"i8"),
            b'b' => return push_basic_type(self, b"bool"),
            b'c' => return push_basic_type(self, b"char"),
            b'd' => return push_basic_type(self, b"f64"),
            b'e' => return push_basic_type(self, b"str"),
            b'f' => return push_basic_type(self, b"f32"),
            b'h' => return push_basic_type(self, b"u8"),
            b'i' => return push_basic_type(self, b"isize"),
            b'j' => return push_basic_type(self, b"usize"),
            b'l' => return push_basic_type(self, b"i32"),
            b'm' => return push_basic_type(self, b"u32"),
            b'n' => return push_basic_type(self, b"i128"),
            b'o' => return push_basic_type(self, b"u128"),
            b's' => return push_basic_type(self, b"i16"),
            b't' => return push_basic_type(self, b"u16"),
            b'u' => return push_basic_type(self, b"()"),
            b'v' => return push_basic_type(self, b"..."),
            b'x' => return push_basic_type(self, b"i64"),
            b'y' => return push_basic_type(self, b"u64"),
            b'z' => return push_basic_type(self, b"!"),

            b'A' => {
                self.out.push(b'[');
                let len = if self.cur().is_ascii_digit() {
                    Some(self.parse_decimal()?)
                } else {
                    None
                };

                self.demangle_type()?;

                if let Some(len) = len {
                    write!(self.out, "; {}", len).unwrap();
                }

                self.out.push(b']');
            }

            b'F' => {
                if self.cur() == b'U' {
                    self.out.extend_from_slice(b"unsafe ");
                    self.pos += 1;
                }

                if self.cur() == b'K' {
                    self.demangle_abi()?;
                    self.out.push(b' ');
                }

                self.out.extend_from_slice(b"fn(");

                let mut any_params = false;
                while self.cur() != b'E' && self.cur() != b'J' {
                    self.demangle_type()?;
                    self.out.push(b',');
                    any_params = true;
                }

                if any_params {
                    debug_assert_eq!(self.out.last(), Some(&b','));
                    self.out.pop();
                }

                self.out.push(b')');

                if self.cur() == b'J' {
                    self.pos += 1;
                    self.out.extend_from_slice(b" -> ");

                    self.demangle_type()?;
                }

                assert_eq!(self.cur(), b'E');
                // Skip the 'E'
                self.pos += 1;
            }

            b'G' => {
                self.demangle_ident()?;
                if self.cur() != b'E' {
                    return Err(format!("While demangling generic parameter name: Expected 'E', found {:?}", self.cur() as char));
                }
                self.pos += 1;
                return Ok(());
            }

            b'N' => {
                self.pos -= 1;
                self.demangle_fully_qualified_name()?;
                // Return because we don't want to add a subst
                return Ok(());
            }

            b'O' => {
                self.out.extend_from_slice(b"*mut ");
                self.demangle_type()?;
            }
            b'P' => {
                self.out.extend_from_slice(b"*const ");
                self.demangle_type()?;
            }
            b'Q' => {
                self.out.extend_from_slice(b"&mut ");
                self.demangle_type()?;
            }
            b'R' => {
                self.out.extend_from_slice(b"&");
                self.demangle_type()?;
            }
            b'S' => {
                self.pos -= 1;
                self.demangle_subst()?;
            }
            b'T' => {
                self.out.push(b'(');
                while self.cur() != b'E' {
                    self.demangle_type()?;
                    self.out.push(b',');
                }
                self.out.pop();
                self.out.push(b')');

                assert_eq!(self.cur(), b'E');
                self.pos += 1;
            }

            other => {
                return Err(format!("expected start of type, found {:?}", other as char));
            }
        }

        self.alloc_subst(subst_start);

        Ok(())
    }

    fn demangle_subst(&mut self) -> Result<(), String> {
        if self.cur() != b'S' {
            return Err(format!("while demangling substitution: expected 'S', found '{}'", self.cur() as char));
        }

        self.pos += 1;

        let index = if self.cur() == b'_' {
            0
        } else {
            self.parse_decimal()? + 1
        };

        assert_eq!(self.cur(), b'_');
        self.pos += 1;

        let range_to_copy = self.dict[index];
        let len = range_to_copy.1 - range_to_copy.0;

        let out_start = self.out.len();
        self.out.resize(out_start + len, 0);

        let (prefix, new_part) = self.out.split_at_mut(out_start);

        new_part.copy_from_slice(&prefix[range_to_copy.0 .. range_to_copy.1]);

        Ok(())
    }

    fn demangle_abi(&mut self) -> Result<(), String> {
        if self.cur() != b'K' {
            return Err(format!("Expected start of <abi> ('K'), found {:?}", self.cur() as char));
        }

        self.pos += 1;

        self.out.extend_from_slice(b"extern \"");

        let tag = self.cur();
        self.out.extend_from_slice(match tag {
            b'c' => b"C",
            c => {
                return Err(format!("Unknown ABI spec {:?}", c as char));
            }
        });

        self.out.push(b'"');
        self.pos += 1;
        Ok(())
    }
}


#[cfg(test)]
mod tests {
    quickcheck! {
        fn demangle_direct(symbol: ::ast::Symbol) -> bool {
            let mut expected = String::new();
            let pretty = symbol.pretty_print(&mut expected);

            let mut uncompressed_mangled = String::new();
            symbol.mangle(&mut uncompressed_mangled);

            let compressed_symbol = ::compress::compress(&symbol);

            let mut compressed_mangled = String::new();
            compressed_symbol.mangle(&mut compressed_mangled);

            let actual = ::direct_demangle::Demangler::demangle(compressed_mangled.as_bytes()).unwrap();

            if actual != expected {
                panic!("expected:     {}\n\
                        actual:       {}\n\
                        compressed:   {}\n\
                        uncompressed: {}\n",
                        expected,
                        actual,
                        compressed_mangled,
                        uncompressed_mangled)
            } else {
                true
            }
        }
    }
}
