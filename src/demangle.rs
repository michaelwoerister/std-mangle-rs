use ast::*;



struct Demangler<'input> {
    pos: usize,

    in: &'input [u8],
    out: Vec<u8>,

    dict: HashMap<Subst, (usize, usize)>,
}

impl Demangler {

    fn demangle_symbol(&mut self) -> Result<(), String> {

        assert!(self.pos == 0);
        if &self.in[0..2] != b"_R" {
            return Err("Not a Rust symbol".to_string())
        }

        self.pos = 2;
        self.demangle_fully_qualified_name()
    }

    fn demangle_fully_qualified_name(&mut self) -> Result<(), String> {
        match self.in[self.pos] {
            b'N' => {
                self.pos += 1;

                self.demangle_name_prefix_with_params()?;

                assert_eq!(self.in[self.pos], b'E');
                self.pos += 1;
                Ok(())
            }
            b'X' => {
                panic!()
            }
            b'S' => {
                self.demangle_subst()
            }
            other => {
                Err(format!("Expected 'N', 'X', or 'S'; found '{}'", other))
            }
        }
    }

    fn demangle_ident(&mut self) -> Result<(), String> {
        panic!()
    }

    fn demangle_name_prefix_with_params(&mut self) -> Result<(), String> {

        let start = self.out.len();

        while self.in[self.pos] != b'E' {
            self.demangle_ident()?;




        }
    }

    fn demangle_name_prefix(&self) -> Result<(), String> {
        if self.in[self.pos] == b'E' {
            return Ok(())
        }


        self.demangle_name_prefix
    }

    fn demangle_subst(&mut self) -> Result<(), String> {
        assert_eq!(self.in[self.pos], b'S');
        self.pos += 1;

        let index = if self.in[self.pos] == b'_' {
            0
            self.pos += 1;
        } else {
            // TODO
            panic!()
        };

        let (start, end) = self.dict[Subst(index)];

        let copy = self.out[start .. end].to_owned();
        self.out.extend(copy);
    }
}

