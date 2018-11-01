
use ast::*;
use std::collections::HashMap;
use std::sync::Arc;

pub(crate) struct Dictionary {
    prefixes: HashMap<Arc<NamePrefix>, Subst>,
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
            qnames: HashMap::new(),
            types: HashMap::new(),
            counter: 0,
        }
    }

    pub(crate) fn to_debug_dictionary(&self) -> Vec<(Subst, String)> {
        let mut items = vec![];

        items.extend(self.prefixes.iter().map(|(k, &v)| (v, format!("{:?}", k))));
        items.extend(self.qnames.iter().map(|(k, &v)| (v, format!("{:?}", k))));
        items.extend(self.types.iter().map(|(k, &v)| (v, format!("{:?}", k))));

        items.sort_by_key(|&(k, _)| k);

        items
    }
}

pub fn compress(symbol: &Symbol) -> Symbol {
    let (compressed, _) = compress_ext(symbol);
    compressed
}

pub(crate) fn compress_ext(symbol: &Symbol) -> (Symbol, Dictionary) {
    let mut dict = Dictionary::new();

    let compressed = Symbol {
        name: compress_fully_qualified_name(&symbol.name, &mut dict),
        // instantiating_crate: compress_name_prefix(&symbol.instantiating_crate, &mut dict),
    };

    if cfg!(debug_assertions) {
        for type_key in dict.types.keys() {
            match **type_key {
                Type::BasicType(_) => {
                    panic!("Found substituted basic type")
                }
                _ => {}
            }
        }
    }

    (compressed, dict)
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
        NamePrefix::InherentImpl { ref self_type } => {
            let new_self_type = compress_type(self_type, dict);

            if Arc::ptr_eq(self_type, &new_self_type) {
                name_prefix.clone()
            } else {
                Arc::new(NamePrefix::InherentImpl {
                    self_type: new_self_type,
                })
            }
        }
        NamePrefix::Node { ref prefix, ref ident } => {
            let new_prefix = compress_name_prefix(prefix, dict);

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

fn compress_fully_qualified_name(qname: &Arc<FullyQualifiedName>,
                                 dict: &mut Dictionary)
                                 -> Arc<FullyQualifiedName> {

    if let Some(&subst) = dict.qnames.get(qname) {
        return Arc::new(FullyQualifiedName::Subst(subst));
    }

    let compressed = match **qname {
        FullyQualifiedName::Name { ref name, ref args } => {
            let new_name = compress_name_prefix(name, dict);
            let new_args = compress_generic_argument_list(args, dict);

            if Arc::ptr_eq(name, &new_name) && new_args.ptr_eq(args) {
                qname.clone()
            } else {
                Arc::new(FullyQualifiedName::Name {
                    name: new_name,
                    args: new_args,
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
            // Return here. We never allocate a substitution for basic types.
            return ty.clone()
        },

        Type::Named(ref name) => {
            if let Some(&subst) = dict.qnames.get(name) {
                return Arc::new(Type::Subst(subst));
            }

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
            abi,
        } => {
            let params = params.iter().map(|t| compress_type(t, dict)).collect();
            let return_type = return_type.as_ref().map(|t| compress_type(t, dict));

            Arc::new(Type::Fn {
                return_type,
                params,
                is_unsafe,
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
