use ast::{NUMERIC_DISAMBIGUATOR_RADIX, SUBST_RADIX};
use charset;
use error::{self, expected};
use int_radix::ascii_digit_to_value;
use parse::EOT;
use std::borrow::Cow;
use std::io::Write;
use std::str;

#[cfg(test)]
use debug::DebugDictionary;

pub struct Demangler<'input> {
    pos: usize,

    verbose: bool,

    input: &'input [u8],
    out: Vec<u8>,
    dict: Vec<(usize, usize)>,
}

impl<'input> Demangler<'input> {
    pub fn demangle(input: &[u8], verbose: bool) -> Result<String, String> {
        let mut state = Demangler::new(input, verbose);

        if let Err(ref msg) = state.demangle_symbol() {
            return Err(state.decorate_error_message(msg));
        }

        String::from_utf8(state.out).map_err(|e| format!("{}", e))
    }

    #[cfg(test)]
    pub fn demangle_debug(
        input: &[u8],
        verbose: bool,
    ) -> (Result<String, String>, DebugDictionary) {
        let mut state = Demangler::new(input, verbose);

        let result = state.demangle_symbol();

        let debug_dict = state
            .dict
            .iter()
            .enumerate()
            .map(|(idx, &(start, end))| {
                (
                    ::ast::Subst(idx as u64),
                    String::from_utf8(state.out[start..end].to_owned()).unwrap(),
                )
            }).collect();

        let debug_dict = DebugDictionary::new(debug_dict);

        match result {
            Ok(()) => (
                String::from_utf8(state.out).map_err(|e| format!("{}", e)),
                debug_dict,
            ),
            Err(ref msg) => (Err(state.decorate_error_message(msg)), debug_dict),
        }
    }

    fn decorate_error_message(&self, msg: &str) -> String {
        format!(
            "Error demangling at position {}: {} - {}",
            self.pos,
            msg,
            str::from_utf8(self.input).unwrap()
        )
    }

    fn new(input: &[u8], verbose: bool) -> Demangler {
        Demangler {
            pos: 0,
            verbose,
            input,
            out: Vec::new(),
            dict: Vec::new(),
        }
    }

    fn demangle_symbol(&mut self) -> Result<(), String> {
        assert!(self.pos == 0);
        if &self.input[0..2] != b"_R" {
            return Err("Not a Rust symbol".to_string());
        }

        self.pos = 2;

        if self.cur().is_ascii_digit() {
            let encoding_version = self.parse_number(10)? + 1;
            return error::version_mismatch(encoding_version, 0);
        }

        // The absolute path
        self.demangle_abs_path()?;

        // The (optional) instantiating crate
        if self.cur() != EOT && self.verbose {
            self.out.extend_from_slice(b" @ ");
            self.demangle_path_prefix()
        } else {
            Ok(())
        }
    }

    fn cur(&self) -> u8 {
        if self.pos < self.input.len() {
            self.input[self.pos]
        } else {
            EOT
        }
    }

    fn alloc_subst(&mut self, start: usize) {
        let end = self.out.len();
        debug_assert!(self.dict.last() != Some(&(start, end)));
        self.dict.push((start, end));
    }

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

    fn demangle_abs_path(&mut self) -> Result<(), String> {
        match self.cur() {
            b'N' => {
                let subst_start = self.out.len();
                self.pos += 1;

                self.demangle_path_prefix()?;

                if self.cur() == b'I' {
                    self.demangle_generic_argument_list()?;
                    self.alloc_subst(subst_start);
                } else {
                    // Don't add a subst because the name prefix already contains the
                    // whole output generated for this name.
                }

                if self.cur() != b'E' {
                    return expected("E", self.cur(), "demangling", "<absolute-path>");
                }

                self.pos += 1;
            }

            b'S' => {
                self.demangle_subst()?;
            }

            c => {
                return expected("NS", c, "demangling", "<absolute-path>");
            }
        }

        Ok(())
    }

    fn parse_opt_numeric_disambiguator(&mut self) -> Result<u64, String> {
        if self.cur() == b's' {
            self.pos += 1;
            let index = if self.cur() == b'_' {
                2
            } else {
                self.parse_number(NUMERIC_DISAMBIGUATOR_RADIX)? + 3
            };

            if self.cur() != b'_' {
                return expected(
                    "_",
                    self.cur(),
                    "demangling",
                    "<underscored-terminated number>",
                );
            }

            self.pos += 1;
            Ok(index)
        } else {
            Ok(1)
        }
    }

    fn demangle_ident(&mut self) -> Result<(), String> {
        if !self.cur().is_ascii_digit() {
            return expected("#", self.cur(), "demangling", "<ident>");
        }

        let ident_len = self.parse_number(10)? as usize;
        let ident_start = self.pos;
        let ident_bytes = &self.input[ident_start..ident_start + ident_len];

        self.pos += ident_len;

        let is_punycode = if self.cur() == b'u' {
            self.pos += 1;
            true
        } else {
            false
        };

        let (is_closure, is_value_ns) = match self.cur() {
            b'C' => {
                self.pos += 1;
                (true, false)
            }
            b'V' => {
                self.pos += 1;
                (false, true)
            }
            _ => (false, false),
        };

        if is_closure {
            self.out.extend_from_slice(b"{closure}");
        } else {
            if is_punycode {
                match charset::decode_punycode_ident(ident_bytes) {
                    Ok(ident) => self.out.extend_from_slice(ident.as_bytes()),
                    Err(e) => return Err(e),
                }
            } else {
                self.out.extend_from_slice(ident_bytes);
            }

            if self.verbose && is_value_ns {
                self.out.push(b'\'');
            }
        }

        let index = self.parse_opt_numeric_disambiguator()?;

        if (index > 1 && self.verbose) || is_closure {
            write!(self.out, "[{}]", index).unwrap();
        }

        Ok(())
    }

    fn demangle_path_prefix(&mut self) -> Result<(), String> {
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

                if self.cur() == b'N' || self.cur() == b'S' {
                    self.out.extend_from_slice(b" as ");
                    self.demangle_abs_path()?;
                }

                self.out.push(b'>');

                let index = self.parse_opt_numeric_disambiguator()?;

                if index > 1 && self.verbose {
                    write!(self.out, "[{}]", index).unwrap();
                }

                true
            }

            b'N' => {
                self.demangle_abs_path()?;
                // The type already added a subst
                false
            }

            c if c.is_ascii_digit() => {
                let crateid_len = self.parse_number(10)? as usize;
                let crateid_start = self.pos;
                let crateid_bytes = &self.input[crateid_start..crateid_start + crateid_len];

                self.pos += crateid_len;

                let name_and_dis = if self.cur() == b'u' {
                    self.pos += 1;
                    match charset::decode_punycode_ident(crateid_bytes) {
                        Ok(ident) => Cow::Owned(ident.into_bytes()),
                        Err(e) => return Err(e),
                    }
                } else {
                    Cow::Borrowed(crateid_bytes)
                };

                if let Some(sep) = name_and_dis.iter().rposition(|&c| c == b'_') {
                    self.out.extend_from_slice(&name_and_dis[..sep]);

                    if self.verbose {
                        self.out.push(b'[');
                        self.out.extend_from_slice(&name_and_dis[sep + 1..]);
                        self.out.push(b']');
                    }
                } else {
                    return Err(format!(
                        "Crate ID '{}' does not contain disambiguator",
                        str::from_utf8(crateid_bytes).unwrap()
                    ));
                };

                true
            }

            c => {
                return expected("SX#", c, "demangling", "<name-prefix>");
            }
        };

        if add_root_to_dict {
            self.alloc_subst(subst_start);
        }

        while self.cur() != EOT && self.cur() != b'E' && self.cur() != b'I' {
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
                    Some(self.parse_number(10)?)
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

                if self.cur() != b'E' {
                    return expected("E", self.cur(), "demangling", "<fn-type>");
                }

                // Skip the 'E'
                self.pos += 1;
            }

            b'G' => {
                self.demangle_ident()?;
                if self.cur() != b'E' {
                    return expected("E", self.cur(), "demangling", "<generic-param-name>");
                }
                self.pos += 1;
            }

            b'N' => {
                self.pos -= 1;
                self.demangle_abs_path()?;
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
                return Ok(());
            }
            b'T' => {
                self.out.push(b'(');
                while self.cur() != b'E' {
                    self.demangle_type()?;
                    self.out.push(b',');
                }
                self.out.pop();
                self.out.push(b')');

                // Skip the 'E'
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
            return expected("S", self.cur(), "demangling", "<substitution>");
        }

        self.pos += 1;

        let index = if self.cur() == b'_' {
            0
        } else {
            self.parse_number(SUBST_RADIX)? as usize + 1
        };

        if self.cur() != b'_' {
            return expected("_", self.cur(), "demangling", "<substitution>");
        }

        self.pos += 1;

        let range_to_copy = if let Some(&index) = self.dict.get(index) {
            index
        } else {
            return Err(format!(
                "Dictionary does not contain substitution index '{}'.",
                index
            ));
        };
        let len = range_to_copy.1 - range_to_copy.0;

        let out_start = self.out.len();
        self.out.resize(out_start + len, 0);

        let (prefix, new_part) = self.out.split_at_mut(out_start);
        new_part.copy_from_slice(&prefix[range_to_copy.0..range_to_copy.1]);

        Ok(())
    }

    fn demangle_abi(&mut self) -> Result<(), String> {
        if self.cur() != b'K' {
            return expected("K", self.cur(), "demangling", "<abi>");
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
