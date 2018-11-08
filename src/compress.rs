
use ast::*;
use std::collections::HashMap;
use std::sync::Arc;
use same::Same;

pub fn compress(symbol: &Symbol) -> Symbol {
    let (compressed, _) = compress_ext(symbol);
    compressed
}

pub(crate) fn compress_ext(symbol: &Symbol) -> (Symbol, Compress) {
    let mut compress = Compress::new();

    let compressed = Symbol {
        name: compress.compress_qname(&symbol.name),
        // instantiating_crate: compress_name_prefix(&symbol.instantiating_crate, &mut dict),
    };

    if cfg!(debug_assertions) {
        compress.sanity_check();
    }

    (compressed, compress)
}

pub struct Compress {
    prefixes: HashMap<Arc<NamePrefix>, Subst>,
    qnames: HashMap<Arc<QName>, Subst>,
    types: HashMap<Arc<Type>, Subst>,
    subst_counter: u64,
}

impl Compress {

    fn new() -> Compress {
        Compress {
            prefixes: HashMap::new(),
            qnames: HashMap::new(),
            types: HashMap::new(),
            subst_counter: 0,
        }
    }

    fn compress_name_prefix(&mut self, name_prefix: &Arc<NamePrefix>) -> Arc<NamePrefix> {

        if let Some(subst) = self.lookup_prefix_subst(name_prefix) {
            return Arc::new(NamePrefix::Subst(subst));
        }

        let compressed = match **name_prefix {
            NamePrefix::CrateId { .. } => {
                // We cannot compress them, just clone the reference to the node
                name_prefix.clone()
            }
            NamePrefix::TraitImpl { ref self_type, ref impled_trait, dis } => {
                let compressed_self_type = self.compress_type(self_type);
                let compressed_impled_trait = self.compress_qname(impled_trait);

                // Don't allocate a new node if it would be the same as the old one
                if compressed_self_type.same_as(self_type) &&
                   compressed_impled_trait.same_as(impled_trait) {
                    name_prefix.clone()
                } else {
                    Arc::new(NamePrefix::TraitImpl {
                        self_type: compressed_self_type,
                        impled_trait: compressed_impled_trait,
                        dis,
                    })
                }
            }
            NamePrefix::InherentImpl { ref self_type } => {
                let compressed_self_type = self.compress_type(self_type);

                // NOTE: We return here and thus don't allocate a substitution.
                //       Compressing `self_type` has already introduced one.
                //
                // Don't allocate a new node if it would be the same as the old one.
                return if compressed_self_type.same_as(self_type) {
                    name_prefix.clone()
                } else {
                    Arc::new(NamePrefix::InherentImpl {
                        self_type: compressed_self_type,
                    })
                };
            }
            NamePrefix::Node { ref prefix, ref ident } => {
                let compressed_prefix = self.compress_name_prefix(prefix);

                // Don't allocate a new node if it would be the same as the old one
                if compressed_prefix.same_as(prefix) {
                    name_prefix.clone()
                } else {
                    Arc::new(NamePrefix::Node {
                        prefix: compressed_prefix,
                        ident: ident.clone(),
                    })
                }
            }
            NamePrefix::Subst(_) => {
                unreachable!()
            }
        };

        self.alloc_subst(name_prefix, |d| &mut d.prefixes);

        compressed
    }

    fn compress_qname(&mut self, qname: &Arc<QName>) -> Arc<QName> {
        if let Some(subst) = self.lookup_qname_subst(qname) {
            return Arc::new(QName::Subst(subst));
        }

        match **qname {
            QName::Name { ref name, ref args } => {
                let compressed_name = self.compress_name_prefix(name);
                let compressed_args = self.compress_generic_argument_list(args);

                if !args.is_empty() {
                    // If there are generic arguments, we add a new substitution in
                    // order to capture them.
                    self.alloc_subst(qname, |d| &mut d.qnames);
                }

                // Don't allocate a new node if it would be the same as the old one
                if compressed_name.same_as(name) && compressed_args.same_as(args) {
                    qname.clone()
                } else {
                    Arc::new(QName::Name {
                        name: compressed_name,
                        args: compressed_args,
                    })
                }
            }
            QName::Subst(_) => {
                unreachable!()
            }
        }
    }

    fn compress_generic_argument_list(&mut self, args: &GenericArgumentList)
                                      -> GenericArgumentList {
        GenericArgumentList(args.iter().map(|t| self.compress_type(t)).collect())
    }

    fn compress_type(&mut self, ty: &Arc<Type>) -> Arc<Type> {

        if let Some(subst) = self.lookup_type_subst(ty) {
            return Arc::new(Type::Subst(subst));
        }

        let compressed = match **ty {
            Type::GenericParam(_) |
            Type::BasicType(_) => {
                // NOTE: We return here as we never allocate a substitution for
                //       basic types or generic parameter names.
                return ty.clone()
            },
            Type::Named(ref name) => {
                // NOTE: Always return here so we don't add something to the dictionary.
                //       Compressing the qname has already taken care of that.
                return dedup(ty, name, self.compress_qname(name), Type::Named)
            }
            Type::Ref(ref inner) => {
                dedup(ty, inner, self.compress_type(inner), Type::Ref)
            }
            Type::RefMut(ref inner) => {
                dedup(ty, inner, self.compress_type(inner), Type::RefMut)
            }
            Type::RawPtrConst(ref inner) => {
                dedup(ty, inner, self.compress_type(inner), Type::RawPtrConst)
            }
            Type::RawPtrMut(ref inner) => {
                dedup(ty, inner, self.compress_type(inner), Type::RawPtrMut)
            }
            Type::Array(opt_size, ref inner) => {
                dedup(ty, inner, self.compress_type(inner), |inner| Type::Array(opt_size, inner))
            }
            Type::Tuple(ref tys) => {
                let compressed_tys: Vec<_> = tys.iter().map(|t| self.compress_type(t)).collect();
                dedup(ty, tys, compressed_tys, Type::Tuple)
            }
            Type::Fn {
                is_unsafe,
                abi,
                ref params,
                ref return_type,
            } => {
                let compressed_params: Vec<_> = params.iter().map(|t| self.compress_type(t)).collect();
                let compressed_return_type = return_type.as_ref().map(|t| self.compress_type(t));

                if compressed_params.same_as(params) &&
                   compressed_return_type.same_as(return_type) {
                    ty.clone()
                } else {
                    Arc::new(Type::Fn {
                        is_unsafe,
                        abi,
                        params: compressed_params,
                        return_type: compressed_return_type,
                    })
                }
            }
            Type::Subst(_) => {
                unreachable!()
            }
        };

        self.alloc_subst(ty, |d| &mut d.types);

        compressed
    }

    fn alloc_subst<T, D>(&mut self, node: &Arc<T>, dict: D)
        where D: FnOnce(&mut Self) -> &mut HashMap<Arc<T>, Subst>,
              T: ::std::hash::Hash + Eq,
    {
        let subst = Subst(self.subst_counter);
        self.subst_counter += 1;
        assert!(dict(self).insert(node.clone(), subst).is_none());
    }

    fn lookup_prefix_subst(&self, name_prefix: &NamePrefix) -> Option<Subst> {
        match *name_prefix {
            NamePrefix::CrateId { .. } |
            NamePrefix::TraitImpl { .. } |
            NamePrefix::Node { .. } => {
                self.prefixes.get(name_prefix).cloned()
            }
            NamePrefix::InherentImpl { ref self_type } => {
                self.lookup_type_subst(self_type)
            }
            NamePrefix::Subst(_) => {
                unreachable!()
            }
        }
    }

    fn lookup_qname_subst(&self, qname: &QName) -> Option<Subst> {
        match *qname {
            QName::Name { ref name, ref args } => {
                if args.is_empty() {
                    self.lookup_prefix_subst(name)
                } else {
                    self.qnames.get(qname).cloned()
                }
            }
            QName::Subst(_) => {
                unreachable!()
            }
        }
    }

    fn lookup_type_subst(&self, ty: &Type) -> Option<Subst> {
        match *ty {
            Type::Named(ref qname) => {
                self.lookup_qname_subst(qname)
            }
            Type::Subst(_) => {
                unreachable!()
            }
            _ => {
                self.types.get(ty).cloned()
            }
        }
    }

    #[cfg(test)]
    pub fn to_debug_dictionary(&self) -> Vec<(Subst, String)> {
        let mut items = vec![];

        items.extend(self.prefixes.iter().map(|(k, &v)| (v, format!("{:?}", k))));
        items.extend(self.qnames.iter().map(|(k, &v)| (v, format!("{:?}", k))));
        items.extend(self.types.iter().map(|(k, &v)| (v, format!("{:?}", k))));

        items.sort_by_key(|&(k, _)| k);

        items
    }

    #[cfg(test)]
    pub fn to_debug_dictionary_pretty(&self) -> Vec<(Subst, String)> {
        let mut items = vec![];

        items.extend(self.prefixes.iter().map(|(k, &v)| {
            let mut pretty = String::new();
            k.pretty_print(&mut pretty);
            (v, pretty)
        }));
        items.extend(self.qnames.iter().map(|(k, &v)| {
            let mut pretty = String::new();
            k.pretty_print(&mut pretty);
            (v, pretty)
        }));
        items.extend(self.types.iter().map(|(k, &v)| {
            let mut pretty = String::new();
            k.pretty_print(&mut pretty);
            (v, pretty)
        }));

        items.sort_by_key(|&(k, _)| k);

        items
    }

    fn sanity_check(&self) {
        // Check basic types never get substituted
        for type_key in self.types.keys() {
            match **type_key {
                Type::BasicType(_) => {
                    panic!("Found substituted basic type")
                }
                _ => {}
            }
        }

        // Check that there are no duplicate substitution indices and no holes
        // in the sequence.
        {
            let mut all_substs: Vec<_> = self.prefixes.values()
                .chain(self.qnames.values())
                .chain(self.types.values())
                .map(|&Subst(idx)| idx)
                .collect();

            if all_substs.len() <= 1 {
                return
            }

            all_substs.sort();

            for i in 1 .. all_substs.len() {
                assert!(all_substs[i - 1] == all_substs[i] - 1);
            }
        }
    }
}

fn dedup<T, I: Same, M>(outer: &Arc<T>,
                        inner: &I,
                        compressed_inner: I,
                        make: M)
                        -> Arc<T>
    where M: FnOnce(I) -> T
{
    if compressed_inner.same_as(inner) {
        outer.clone()
    } else {
        Arc::new(make(compressed_inner))
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
