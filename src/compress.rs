use ast::*;
use same::Same;
use std::collections::HashMap;
use std::sync::Arc;

#[cfg(test)]
use debug::DebugDictionary;

pub fn compress_ext(symbol: &Symbol) -> (Symbol, Compress) {
    let mut compress = Compress::new();

    let compressed = Symbol {
        name: compress.compress_abs_path(&symbol.name),
        instantiating_crate: symbol
            .instantiating_crate
            .as_ref()
            .map(|ic| compress.compress_path_prefix(ic)),
    };

    #[cfg(test)]
    {
        compress.sanity_check();
    }

    (compressed, compress)
}

pub struct Compress {
    prefixes: HashMap<Arc<PathPrefix>, Subst>,
    abs_paths: HashMap<Arc<AbsolutePath>, Subst>,
    types: HashMap<Arc<Type>, Subst>,
    subst_counter: u64,
}

impl Compress {
    fn new() -> Compress {
        Compress {
            prefixes: HashMap::new(),
            abs_paths: HashMap::new(),
            types: HashMap::new(),
            subst_counter: 0,
        }
    }

    fn compress_path_prefix(&mut self, path_prefix: &Arc<PathPrefix>) -> Arc<PathPrefix> {
        if let Some(subst) = self.lookup_prefix_subst(path_prefix) {
            return Arc::new(PathPrefix::Subst(subst));
        }

        let compressed = match **path_prefix {
            PathPrefix::CrateId { .. } => {
                // We cannot compress them, just clone the reference to the node
                path_prefix.clone()
            }
            PathPrefix::TraitImpl {
                ref self_type,
                ref impled_trait,
                dis,
            } => {
                let compressed_self_type = self.compress_type(self_type);
                let compressed_impled_trait = self.compress_abs_path(impled_trait);

                // Don't allocate a new node if it would be the same as the old one
                if compressed_self_type.same_as(self_type)
                    && compressed_impled_trait.same_as(impled_trait)
                {
                    path_prefix.clone()
                } else {
                    Arc::new(PathPrefix::TraitImpl {
                        self_type: compressed_self_type,
                        impled_trait: compressed_impled_trait,
                        dis,
                    })
                }
            }
            PathPrefix::InherentImpl { ref self_type } => {
                let compressed_self_type = self.compress_type(self_type);

                // NOTE: We return here and thus don't allocate a substitution.
                //       Compressing `self_type` has already introduced one.
                //
                // Don't allocate a new node if it would be the same as the old one.
                return if compressed_self_type.same_as(self_type) {
                    path_prefix.clone()
                } else {
                    Arc::new(PathPrefix::InherentImpl {
                        self_type: compressed_self_type,
                    })
                };
            }
            PathPrefix::Node {
                ref prefix,
                ref ident,
            } => {
                let compressed_prefix = self.compress_path_prefix(prefix);

                // Don't allocate a new node if it would be the same as the old one
                if compressed_prefix.same_as(prefix) {
                    path_prefix.clone()
                } else {
                    Arc::new(PathPrefix::Node {
                        prefix: compressed_prefix,
                        ident: ident.clone(),
                    })
                }
            }
            PathPrefix::Subst(_) => unreachable!(),
        };

        self.alloc_subst(path_prefix, |d| &mut d.prefixes);

        compressed
    }

    fn compress_abs_path(&mut self, abs_path: &Arc<AbsolutePath>) -> Arc<AbsolutePath> {
        if let Some(subst) = self.lookup_abs_path_subst(abs_path) {
            return Arc::new(AbsolutePath::Subst(subst));
        }

        match **abs_path {
            AbsolutePath::Path { ref name, ref args } => {
                let compressed_name = self.compress_path_prefix(name);
                let compressed_args = self.compress_generic_argument_list(args);

                if !args.is_empty() {
                    // If there are generic arguments, we add a new substitution in
                    // order to capture them.
                    self.alloc_subst(abs_path, |d| &mut d.abs_paths);
                }

                // Don't allocate a new node if it would be the same as the old one
                if compressed_name.same_as(name) && compressed_args.same_as(args) {
                    abs_path.clone()
                } else {
                    Arc::new(AbsolutePath::Path {
                        name: compressed_name,
                        args: compressed_args,
                    })
                }
            }
            AbsolutePath::Subst(_) => unreachable!(),
        }
    }

    fn compress_generic_argument_list(
        &mut self,
        args: &GenericArgumentList,
    ) -> GenericArgumentList {
        GenericArgumentList(args.iter().map(|t| self.compress_type(t)).collect())
    }

    fn compress_type(&mut self, ty: &Arc<Type>) -> Arc<Type> {
        if let Some(subst) = self.lookup_type_subst(ty) {
            return Arc::new(Type::Subst(subst));
        }

        let compressed = match **ty {
            Type::BasicType(_) => {
                // NOTE: We return here as we never allocate a substitution for
                //       basic types.
                return ty.clone();
            }
            Type::Named(ref name) => {
                // NOTE: Always return here so we don't add something to the dictionary.
                //       Compressing the abs_path has already taken care of that.
                return dedup(ty, name, self.compress_abs_path(name), Type::Named);
            }
            Type::Ref(ref inner) => dedup(ty, inner, self.compress_type(inner), Type::Ref),
            Type::RefMut(ref inner) => dedup(ty, inner, self.compress_type(inner), Type::RefMut),
            Type::RawPtrConst(ref inner) => {
                dedup(ty, inner, self.compress_type(inner), Type::RawPtrConst)
            }
            Type::RawPtrMut(ref inner) => {
                dedup(ty, inner, self.compress_type(inner), Type::RawPtrMut)
            }
            Type::Array(opt_size, ref inner) => {
                dedup(ty, inner, self.compress_type(inner), |inner| {
                    Type::Array(opt_size, inner)
                })
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
                let compressed_params: Vec<_> =
                    params.iter().map(|t| self.compress_type(t)).collect();
                let compressed_return_type = return_type.as_ref().map(|t| self.compress_type(t));

                if compressed_params.same_as(params) && compressed_return_type.same_as(return_type)
                {
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
            Type::GenericParam(_) => ty.clone(),
            Type::Subst(_) => unreachable!(),
        };

        self.alloc_subst(ty, |d| &mut d.types);

        compressed
    }

    fn alloc_subst<T, D>(&mut self, node: &Arc<T>, dict: D)
    where
        D: FnOnce(&mut Self) -> &mut HashMap<Arc<T>, Subst>,
        T: ::std::hash::Hash + Eq,
    {
        let subst = Subst(self.subst_counter);
        self.subst_counter += 1;
        assert!(dict(self).insert(node.clone(), subst).is_none());
    }

    fn lookup_prefix_subst(&self, path_prefix: &PathPrefix) -> Option<Subst> {
        match *path_prefix {
            PathPrefix::CrateId { .. } | PathPrefix::TraitImpl { .. } | PathPrefix::Node { .. } => {
                self.prefixes.get(path_prefix).cloned()
            }
            PathPrefix::InherentImpl { ref self_type } => self.lookup_type_subst(self_type),
            PathPrefix::Subst(_) => unreachable!(),
        }
    }

    fn lookup_abs_path_subst(&self, abs_path: &AbsolutePath) -> Option<Subst> {
        match *abs_path {
            AbsolutePath::Path { ref name, ref args } => {
                if args.is_empty() {
                    self.lookup_prefix_subst(name)
                } else {
                    self.abs_paths.get(abs_path).cloned()
                }
            }
            AbsolutePath::Subst(_) => unreachable!(),
        }
    }

    fn lookup_type_subst(&self, ty: &Type) -> Option<Subst> {
        match *ty {
            Type::Named(ref abs_path) => self.lookup_abs_path_subst(abs_path),
            Type::Subst(_) => unreachable!(),
            _ => self.types.get(ty).cloned(),
        }
    }
}

fn dedup<T, I: Same, M>(outer: &Arc<T>, inner: &I, compressed_inner: I, make: M) -> Arc<T>
where
    M: FnOnce(I) -> T,
{
    if compressed_inner.same_as(inner) {
        outer.clone()
    } else {
        Arc::new(make(compressed_inner))
    }
}

#[cfg(test)]
impl Compress {
    pub fn to_debug_dictionary(&self) -> DebugDictionary {
        use ast_demangle::AstDemangle;

        let mut items = vec![];

        items.extend(self.prefixes.iter().map(|(k, &v)| (v, k.demangle(true))));
        items.extend(self.abs_paths.iter().map(|(k, &v)| (v, k.demangle(true))));
        items.extend(self.types.iter().map(|(k, &v)| (v, k.demangle(true))));

        DebugDictionary::new(items)
    }

    fn sanity_check(&self) {
        // Check basic types never get substituted
        for type_key in self.types.keys() {
            match **type_key {
                Type::BasicType(_) => panic!("Found substituted basic type"),
                _ => {}
            }
        }

        // Check that there are no duplicate substitution indices and no holes
        // in the sequence.
        {
            let mut all_substs: Vec<_> = self
                .prefixes
                .values()
                .chain(self.abs_paths.values())
                .chain(self.types.values())
                .map(|&Subst(idx)| idx)
                .collect();

            if all_substs.len() <= 1 {
                return;
            }

            all_substs.sort();

            for i in 1..all_substs.len() {
                assert!(all_substs[i - 1] == all_substs[i] - 1);
            }
        }
    }
}
