
use ast::*;

pub fn print_dictionary<I>(entries: I)
    where I: Iterator<Item=(Subst, String)>
{
    let mut items: Vec<_> = entries.collect();

    items.sort_by_key(|&(s, _)| s);

    for (k, v) in items {
        println!("{:6>} {}", k, v);
    }
}


pub fn compare_dictionaries(d1: &[(Subst, String)], d2: &[(Subst, String)]) {

    if d1.len() != d2.len() {
        println!("dictionaries differ in length ({} vs {})", d1.len(), d2.len());
    } else {
        println!("both dictionaries have {} entries", d1.len());
    }

    for i in 0 .. d1.len() {
        if d1[i] != d2[i] {
            println!("Element {} differs:", i);
            println!(" (1) {:?} => {}", d1[i].0, d1[i].1);
            println!(" (2) {:?} => {}", d2[i].0, d2[i].1);
            return
        }
    }

    println!("dictionaries equal!");

    // for (k, v) in d1 {
    //     println!("{:?} => {}", k, v);
    // }
}

