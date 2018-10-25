
use std::fmt::{Display, Error, Formatter};
use ast::*;

impl Display for Ident {
    fn fmt(&self, out: &mut Formatter) -> Result<(), Error> {
        // TODO: non-symbol characters

        let len = self.ident.len();

        if len == 0 {
            return Err(Error);
        }

        write!(out, "{}{}", len, self.ident)?;
        // if let Some(namespace) = self.tag {
        //     write!(out, "{}", namespace)?;
        // }

        Ok(())
    }
}


impl Display for Subst {
    fn fmt(&self, out: &mut Formatter) -> Result<(), Error> {
        if self.0 == 0 {
            write!(out, "S_")
        } else {
            write!(out, "S{}_", self.0 - 1)
        }
    }
}

// impl Display for NamePrefix {
//     fn fmt(&self, out: &mut Formatter) -> Result<(), Error> {
//         match *self {
//             NamePrefix::CrateId { .. } => {
//                 // let len = name.len() + dis.len() + 1;
//                 // write!(out, "{}{}_{}", len, name, dis)
//                 panic!("TODO")
//             }
//             NamePrefix::Node { .. } => { //ref prefix, ref _ident, ref _args } => {
//                 panic!("TODO")
//                 // write!(out, "{}{}", prefix, ident)
//             }
//             NamePrefix::Subst(subst) => {
//                 subst.fmt(out)
//             }
//         }
//     }
// }

// #[cfg(test)]
// mod tests {
//     use ast::*;
//     use std::fmt::Display;

//     fn test<AST: Display>(ast: AST, expected: &str) {
//         assert_eq!(&format!("{}", ast), expected);
//     }

//     #[test]
//     fn test_asscii_ident() {
//         test(Ident::new("abc", None), "3abc");
//         test(Ident::new("abc", Some('F')), "3abcF");
//     }

//     #[test]
//     #[should_panic]
//     fn test_empty_ident() {
//         format!("{}", Ident::new("", None));
//     }

//     #[test]
//     #[should_panic]
//     fn test_empty_ident_w_ns() {
//         format!("{}", Ident::new("", Some('F')));
//     }

//     #[test]
//     fn test_name_prefix_crate_id() {
//         test(NamePrefix::crate_id("abc", "xyz"), "7abc_xyz");
//     }

//     #[test]
//     fn test_name_prefix_node() {
//         test(NamePrefix::node(
//                 NamePrefix::crate_id("abc", "xyz"),
//                 Ident::new("quux", Some('F'))),
//              "7abc_xyz4quuxF");
//     }

//     #[test]
//     fn test_name_prefix_node2() {
//         test(NamePrefix::node(
//                 NamePrefix::node(
//                     NamePrefix::crate_id("abc", "xyz"),
//                     Ident::new("quux", Some('F'))),
//                 Ident::new("foo", None)),
//              "7abc_xyz4quuxF3foo");
//     }

//     #[test]
//     fn test_name_prefix_subst() {
//         test(NamePrefix::node(NamePrefix::subst(4), Ident::new("quux", Some('F'))),
//              "S3_4quuxF");
//     }
// }

