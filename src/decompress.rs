

use ast::*;
use std::collections::HashMap;
use std::sync::Arc;

pub struct Decompress {
    qnames: HashMap<Subst, Arc<FullyQualifiedName>>,
    name_prefixes: HashMap<Subst, Arc<NamePrefix>>,
    types: HashMap<Subst, Arc<Type>>,

    subst_counter: usize,
}


impl Decompress {

    pub fn new() -> Decompress {
        Decompress {
            qnames: HashMap::new(),
            name_prefixes: HashMap::new(),
            types: HashMap::new(),
            subst_counter: 0,
        }
    }

    pub fn decompress(symbol: &Symbol) -> Symbol {

        let decompressed_qname = Decompress::new()
            .decompress_fully_qualified_name(&symbol.name);

        Symbol {
            name: decompressed_qname,
        }
    }

    pub(crate) fn decompress_symbol(&mut self, symbol: &Symbol) -> Symbol {

        let decompressed_qname = self.decompress_fully_qualified_name(&symbol.name);

        Symbol {
            name: decompressed_qname,
        }
    }

    pub(crate) fn to_debug_dictionary(&self) -> Vec<(Subst, String)> {
        let mut items = vec![];

        items.extend(self.name_prefixes.iter().map(|(&k, v)| (k, format!("{:?}", v))));
        items.extend(self.qnames.iter().map(|(&k, v)| (k, format!("{:?}", v))));
        items.extend(self.types.iter().map(|(&k, v)| (k, format!("{:?}", v))));

        items.sort_by_key(|&(k, _)| k);

        items
    }

    fn alloc_subst<T, D>(&mut self, node: &Arc<T>, dict: D)
        where D: FnOnce(&mut Self) -> &mut HashMap<Subst, Arc<T>>,
              T: ::std::hash::Hash + Eq,
    {
        let subst = Subst(self.subst_counter);
        self.subst_counter += 1;
        dict(self).insert(subst, node.clone());
    }

    fn decompress_fully_qualified_name(&mut self,
                                       qname: &Arc<FullyQualifiedName>)
                                       -> Arc<FullyQualifiedName> {
        match **qname {
            FullyQualifiedName::Name { ref name, ref args } => {
                let new_name_prefix = self.decompress_name_prefix(name);
                let decompressed_args = self.decompress_generic_parameter_list(args);

                let decompressed = if Arc::ptr_eq(name, &new_name_prefix) &&
                                      decompressed_args.ptr_eq(args) {
                    qname.clone()
                } else {
                    Arc::new(FullyQualifiedName::Name {
                        name: new_name_prefix,
                        args: decompressed_args,
                    })
                };

                self.alloc_subst(&decompressed, |this| &mut this.qnames);
                decompressed
            }
            FullyQualifiedName::Subst(ref subst) => {
                self.qnames[subst].clone()
            }
        }
    }

    fn decompress_name_prefix(&mut self,
                              name_prefix: &Arc<NamePrefix>)
                              -> Arc<NamePrefix> {
        let decompressed = match **name_prefix {
            NamePrefix::CrateId { .. } => {
                name_prefix.clone()
            }
            NamePrefix::TraitImpl { ref self_type, ref impled_trait } => {
                let decompressed_self_type = self.decompress_type(self_type);
                let decompressed_impled_trait = self.decompress_fully_qualified_name(impled_trait);

                if Arc::ptr_eq(self_type, &decompressed_self_type) &&
                   Arc::ptr_eq(impled_trait, &decompressed_impled_trait) {
                    name_prefix.clone()
                } else {
                    Arc::new(NamePrefix::TraitImpl {
                        self_type: decompressed_self_type,
                        impled_trait: decompressed_impled_trait,
                    })
                }
            }
            NamePrefix::InherentImpl { ref self_type } => {
                let decompressed_self_type = self.decompress_type(self_type);

                if Arc::ptr_eq(self_type, &decompressed_self_type) {
                    name_prefix.clone()
                } else {
                    Arc::new(NamePrefix::InherentImpl {
                        self_type: decompressed_self_type,
                    })
                }
            }
            NamePrefix::Node { ref prefix, ref ident } => {
                let decompressed_prefix = self.decompress_name_prefix(prefix);

                if Arc::ptr_eq(prefix, &decompressed_prefix) {
                    name_prefix.clone()
                } else {
                    Arc::new(NamePrefix::Node {
                        prefix: decompressed_prefix,
                        ident: ident.clone(),
                    })
                }
            }
            NamePrefix::Subst(ref subst) => {
                // Return here! Don't add anything to the dictionary.
                return self.name_prefixes[subst].clone();
            }
        };

        self.alloc_subst(&decompressed, |this| &mut this.name_prefixes);
        decompressed
    }

    fn decompress_generic_parameter_list(&mut self,
                                         compressed: &GenericArgumentList)
                                         -> GenericArgumentList
    {
        let decompressed_params = compressed
            .params
            .iter()
            .map(|t| self.decompress_type(t))
            .collect();

        GenericArgumentList {
            params: decompressed_params
        }
    }

    fn decompress_type(&mut self, compressed: &Arc<Type>) -> Arc<Type> {

        let decompressed = match **compressed {
            Type::BasicType(_) => {
                // Exit here!
                return compressed.clone();
            }
            Type::Ref(ref compressed_inner) => {
                let decompressed_inner = self.decompress_type(compressed_inner);

                if Arc::ptr_eq(compressed_inner, &decompressed_inner) {
                    compressed.clone()
                } else {
                    Arc::new(Type::Ref(decompressed_inner))
                }
            }
            Type::RefMut(ref compressed_inner) => {
                let decompressed_inner = self.decompress_type(compressed_inner);

                if Arc::ptr_eq(compressed_inner, &decompressed_inner) {
                    compressed.clone()
                } else {
                    Arc::new(Type::RefMut(decompressed_inner))
                }
            }
            Type::RawPtrConst(ref compressed_inner) => {
                let decompressed_inner = self.decompress_type(compressed_inner);

                if Arc::ptr_eq(compressed_inner, &decompressed_inner) {
                    compressed.clone()
                } else {
                    Arc::new(Type::RawPtrConst(decompressed_inner))
                }
            }
            Type::RawPtrMut(ref compressed_inner) => {
                let decompressed_inner = self.decompress_type(compressed_inner);

                if Arc::ptr_eq(compressed_inner, &decompressed_inner) {
                    compressed.clone()
                } else {
                    Arc::new(Type::RawPtrMut(decompressed_inner))
                }
            }
            Type::Array(opt_size, ref compressed_inner) => {
                let decompressed_inner = self.decompress_type(compressed_inner);

                if Arc::ptr_eq(compressed_inner, &decompressed_inner) {
                    compressed.clone()
                } else {
                    Arc::new(Type::Array(opt_size, decompressed_inner))
                }
            }
            Type::Tuple(ref compressed_components) => {
                let decompressed_components: Vec<_> = compressed_components
                    .iter()
                    .map(|t| self.decompress_type(t))
                    .collect();

                if decompressed_components.iter().zip(compressed_components.iter())
                    .all(|(a, b)| Arc::ptr_eq(a, b)) {
                    compressed.clone()
                } else {
                    Arc::new(Type::Tuple(decompressed_components))
                }
            }
            Type::Named(ref qname) => {
                let decompressed_qname = self.decompress_fully_qualified_name(qname);

                // Exit here!
                return if Arc::ptr_eq(qname, &decompressed_qname) {
                    compressed.clone()
                } else {
                    Arc::new(Type::Named(decompressed_qname))
                };
            }
            Type::GenericParam(_) => {
                return compressed.clone();
            }
            Type::Fn { is_unsafe, abi, ref return_type, ref params } => {
                let decompressed_params: Vec<_> = params
                    .iter()
                    .map(|t| self.decompress_type(t))
                    .collect();

                let decompressed_return_type = return_type.as_ref()
                                                          .map(|t| self.decompress_type(t));

                let return_types_same = match (return_type, &decompressed_return_type) {
                    (Some(ref a), Some(ref b)) => Arc::ptr_eq(a, b),
                    (None, None) => true,
                    _ => unreachable!(),
                };

                if return_types_same &&
                    decompressed_params.iter().zip(params.iter())
                        .all(|(a, b)| Arc::ptr_eq(a, b)) {
                    compressed.clone()
                } else {
                    Arc::new(Type::Fn {
                        is_unsafe,
                        abi,
                        return_type: decompressed_return_type,
                        params: decompressed_params,
                    })
                }
            }
            Type::Subst(ref subst) => {
                // Review with respect to Named variant / parsing? compression?
                // return self.types[subst].clone();

                return if let Some(t) = self.types.get(subst) {
                    t.clone()
                } else {
                    Arc::new(Type::Named(self.qnames[subst].clone()))
                };
            }
        };

        self.alloc_subst(&decompressed, |this| &mut this.types);

        decompressed
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use debug;

    quickcheck! {
        fn compress_decompress(symbol: Symbol) -> bool {
            let (compressed, c_dict) = ::compress::compress_ext(&symbol);

            let mut d = Decompress::new();
            let decompressed = d.decompress_symbol(&compressed);

            if symbol != decompressed {
                debug::compare_dictionaries(
                    &d.to_debug_dictionary(),
                    &c_dict.to_debug_dictionary(),
                );

                panic!("original:     {:?}\n\
                        decompressed: {:?}\n\
                        compressed:   {:?}\n",
                symbol,
                decompressed,
                compressed)
            }

            true
        }
    }
}
