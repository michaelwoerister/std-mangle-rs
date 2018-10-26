
use ast::*;
use std::collections::HashMap;
use std::sync::Arc;

struct Dictionary {
    prefixes: HashMap<Arc<NamePrefix>, Subst>,
    prefixes_with_params: HashMap<Arc<NamePrefixWithParams>, Subst>,
    qnames: HashMap<Arc<FullyQualifiedName>, Subst>,
    types: HashMap<Arc<Type>, Subst>,

    counter: usize,
}

impl Dictionary {
    fn alloc_subst<T, D>(&mut self, node: &Arc<T>, dict: D)
        where D: FnOnce(&mut Self) -> &mut HashMap<Arc<T>, Subst>,
              T: ::std::hash::Hash + Eq,
    {
        let subst = Subst(self.counter);
        self.counter += 1;
        dict(self).insert(node.clone(), subst);
    }

    fn new() -> Dictionary {
        Dictionary {
            prefixes: HashMap::new(),
            prefixes_with_params: HashMap::new(),
            qnames: HashMap::new(),
            types: HashMap::new(),
            counter: 0,
        }
    }
}

pub fn compress(symbol: &Symbol) -> Symbol {

    let mut dict = Dictionary::new();

    Symbol {
        name: compress_fully_qualified_name(&symbol.name, &mut dict),
        instantiating_crate: compress_name_prefix(&symbol.instantiating_crate, &mut dict),
    }
}

fn compress_name_prefix(name_prefix: &Arc<NamePrefix>, dict: &mut Dictionary) -> Arc<NamePrefix> {

    if let Some(&subst) = dict.prefixes.get(name_prefix) {
        return Arc::new(NamePrefix::Subst(subst));
    }

    let compressed = match **name_prefix {
        NamePrefix::CrateId { .. } => {
            name_prefix.clone()
        }
        NamePrefix::TraitImpl { ref self_type, ref impled_trait } => {
            let new_self_type = compress_type(self_type, dict);
            let new_impled_trait = compress_fully_qualified_name(impled_trait, dict);

            if Arc::ptr_eq(self_type, &new_self_type) &&
               Arc::ptr_eq(impled_trait, &new_impled_trait) {
                name_prefix.clone()
            } else {
                Arc::new(NamePrefix::TraitImpl {
                    self_type: new_self_type,
                    impled_trait: new_impled_trait,
                })
            }
        }
        NamePrefix::Node { ref prefix, ref ident } => {
            let new_prefix = compress_name_prefix_with_params(prefix, dict);

            if Arc::ptr_eq(&new_prefix, prefix) {
                name_prefix.clone()
            } else {
                Arc::new(NamePrefix::Node {
                    prefix: new_prefix,
                    ident: ident.clone(),
                })
            }
        }
        NamePrefix::Subst(_) => {
            unreachable!()
        }
    };

    dict.alloc_subst(name_prefix, |d| &mut d.prefixes);

    compressed
}


fn compress_name_prefix_with_params(name_prefix: &Arc<NamePrefixWithParams>,
                                    dict: &mut Dictionary)
                                    -> Arc<NamePrefixWithParams> {

    if let Some(&subst) = dict.prefixes_with_params.get(name_prefix) {
        return Arc::new(NamePrefixWithParams::Subst(subst));
    }

    let (compressed, new_dict_entry) = match **name_prefix {
        NamePrefixWithParams::Node { ref prefix, ref args, } => {
            let new_prefix = compress_name_prefix(prefix, dict);

            let new_args = compress_generic_argument_list(args, dict);

            let compressed = if Arc::ptr_eq(prefix, &new_prefix) && args == &new_args {
                name_prefix.clone()
            } else {
                Arc::new(NamePrefixWithParams::Node {
                    prefix: new_prefix,
                    args: new_args,
                })
            };

            (compressed, !args.params.is_empty())
        }
        NamePrefixWithParams::Subst(_) => {
            unreachable!()
        }
    };

    if new_dict_entry {
        dict.alloc_subst(name_prefix, |d| &mut d.prefixes_with_params);
    }

    compressed
}


fn compress_fully_qualified_name(qname: &Arc<FullyQualifiedName>,
                                 dict: &mut Dictionary)
                                 -> Arc<FullyQualifiedName> {

    if let Some(&subst) = dict.qnames.get(qname) {
        return Arc::new(FullyQualifiedName::Subst(subst));
    }

    let compressed = match **qname {
        FullyQualifiedName::Name { ref name } => {
            let new_name = compress_name_prefix_with_params(name, dict);

            if Arc::ptr_eq(name, &new_name) {
                qname.clone()
            } else {
                Arc::new(FullyQualifiedName::Name {
                    name: new_name,
                })
            }
        }
        FullyQualifiedName::Subst(_) => {
            unreachable!()
        }
    };

    dict.alloc_subst(qname, |d| &mut d.qnames);

    compressed
}

fn compress_generic_argument_list(args: &GenericArgumentList, dict: &mut Dictionary) -> GenericArgumentList {
    GenericArgumentList {
        params: args.params.iter().map(|t| compress_type(t, dict)).collect(),
    }
}

#[allow(unused)]
fn compress_param_bound(b: &Arc<ParamBound>, dict: &mut Dictionary) -> Arc<ParamBound> {
    Arc::new(ParamBound {
        param_name: b.param_name.clone(),
        bounds: b.bounds.iter().map(|t| compress_type(t, dict)).collect()
    })
}

fn compress_type(ty: &Arc<Type>, dict: &mut Dictionary) -> Arc<Type> {

    if let Some(&subst) = dict.types.get(ty) {
        return Arc::new(Type::Subst(subst));
    }

    let compressed = match **ty {
        Type::GenericParam(_) |
        Type::BasicType(_) => {
            return ty.clone()
        },

        Type::Named(ref name) => {
            // Always return here so we don't add something to the dictionary.
            return compress_dedup(ty, name, dict, compress_fully_qualified_name, Type::Named)
        }

        Type::Ref(ref inner) => {
            compress_dedup(ty, inner, dict, compress_type, Type::Ref)
        }
        Type::RefMut(ref inner) => {
            compress_dedup(ty, inner, dict, compress_type, Type::RefMut)
        }
        Type::RawPtrConst(ref inner) => {
            compress_dedup(ty, inner, dict, compress_type, Type::RawPtrConst)
        }
        Type::RawPtrMut(ref inner) => {
            compress_dedup(ty, inner, dict, compress_type, Type::RawPtrMut)
        }
        Type::Array(opt_size, ref inner) => {
            compress_dedup(ty, inner, dict, compress_type, |inner| Type::Array(opt_size, inner))
        }
        Type::Tuple(ref tys) => {
            let new_tys: Vec<_> = tys.iter().map(|t| {
                compress_type(t, dict)
            }).collect();

            if new_tys.iter().zip(tys.iter()).all(|(t1, t2)| Arc::ptr_eq(t1, t2)) {
                ty.clone()
            } else {
                Arc::new(Type::Tuple(new_tys))
            }
        }
        Type::Fn {
            ref return_type,
            ref params,
            is_unsafe,
            is_variadic,
            abi,
        } => {
            let return_type = compress_type(return_type, dict);
            let params = params.iter().map(|t| compress_type(t, dict)).collect();

            Arc::new(Type::Fn {
                return_type,
                params,
                is_unsafe,
                is_variadic,
                abi,
            })
        }

        Type::Subst(_) => {
            unreachable!()
        }
    };

    dict.alloc_subst(ty, |d| &mut d.types);

    compressed
}

fn compress_dedup<T, T2, C, M>(val: &Arc<T2>,
                               inner: &Arc<T>,
                               dict: &mut Dictionary,
                               compress: C,
                               make: M)
                               -> Arc<T2>
    where C: FnOnce(&Arc<T>, &mut Dictionary) -> Arc<T>,
          M: FnOnce(Arc<T>) -> T2
{
    let compressed = compress(inner, dict);

    if Arc::ptr_eq(inner, &compressed) {
        val.clone()
    } else {
        Arc::new(make(compressed))
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    quickcheck! {
        fn compress_does_not_crash(symbol: Symbol) -> bool {
            compress(&symbol);
            true
        }
    }
}
