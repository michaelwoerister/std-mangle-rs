//! An alternative implementation of the compression algorithm. It determines
//! node equivalence via lossless demangling instead of AST structure.
//! This implementation is slower and meant mainly as a sanity check for the
//! regular implementation.

use ast::*;
use ast_demangle::AstDemangle;
use std::collections::HashMap;
use std::sync::Arc;

#[cfg(test)]
use debug::DebugDictionary;

pub struct CompressAlt {
    dict: HashMap<String, Subst>,
    subst_counter: u64,
}

pub fn compress_ext(symbol: &Symbol) -> (Symbol, CompressAlt) {
    let mut compress = CompressAlt::new();

    let compressed = Symbol {
        name: compress.compress_abs_path(&symbol.name),
        instantiating_crate: symbol
            .instantiating_crate
            .as_ref()
            .map(|ic| compress.compress_path_prefix(ic)),
    };

    (compressed, compress)
}

impl CompressAlt {
    fn new() -> CompressAlt {
        CompressAlt {
            dict: HashMap::new(),
            subst_counter: 0,
        }
    }

    fn compress_path_prefix(&mut self, name_prefix: &Arc<PathPrefix>) -> Arc<PathPrefix> {
        let demangled = name_prefix.demangle(true);

        if let Some(&subst) = self.dict.get(&demangled) {
            return Arc::new(PathPrefix::Subst(subst));
        }

        let compressed = match **name_prefix {
            PathPrefix::CrateId { .. } => name_prefix.clone(),
            PathPrefix::TraitImpl {
                ref self_type,
                ref impled_trait,
                dis,
            } => Arc::new(PathPrefix::TraitImpl {
                self_type: self.compress_type(self_type),
                impled_trait: impled_trait.as_ref().map(|t| self.compress_abs_path(t)),
                dis,
            }),
            PathPrefix::Node {
                ref prefix,
                ref ident,
            } => Arc::new(PathPrefix::Node {
                prefix: self.compress_path_prefix(prefix),
                ident: ident.clone(),
            }),
            PathPrefix::Subst(_) => unreachable!(),
        };

        self.alloc_subst(demangled);

        compressed
    }

    fn compress_abs_path(&mut self, abs_path: &Arc<AbsolutePath>) -> Arc<AbsolutePath> {
        let demangled = abs_path.demangle(true);

        if let Some(&subst) = self.dict.get(&demangled) {
            return Arc::new(AbsolutePath::Subst(subst));
        }

        let compressed = match **abs_path {
            AbsolutePath::Path { ref name, ref args } => Arc::new(AbsolutePath::Path {
                name: self.compress_path_prefix(name),
                args: self.compress_generic_argument_list(args),
            }),
            AbsolutePath::Subst(_) => unreachable!(),
        };

        self.alloc_subst(demangled);
        compressed
    }

    fn compress_generic_argument_list(
        &mut self,
        args: &GenericArgumentList,
    ) -> GenericArgumentList {
        GenericArgumentList(args.iter().map(|t| self.compress_type(t)).collect())
    }

    fn compress_type(&mut self, ty: &Arc<Type>) -> Arc<Type> {
        let demangled = ty.demangle(true);

        if let Some(&subst) = self.dict.get(&demangled) {
            return Arc::new(Type::Subst(subst));
        }

        let compressed = match **ty {
            Type::BasicType(_) => {
                // NOTE: We return here as we never allocate a substitution for
                //       basic types.
                return ty.clone();
            }
            Type::Named(ref name) => Type::Named(self.compress_abs_path(name)),
            Type::Ref(ref inner) => Type::Ref(self.compress_type(inner)),
            Type::RefMut(ref inner) => Type::RefMut(self.compress_type(inner)),
            Type::RawPtrConst(ref inner) => Type::RawPtrConst(self.compress_type(inner)),
            Type::RawPtrMut(ref inner) => Type::RawPtrMut(self.compress_type(inner)),
            Type::Array(opt_size, ref inner) => Type::Array(opt_size, self.compress_type(inner)),
            Type::Tuple(ref tys) => {
                Type::Tuple(tys.iter().map(|t| self.compress_type(t)).collect())
            }
            Type::Fn {
                is_unsafe,
                abi,
                ref params,
                ref return_type,
            } => Type::Fn {
                is_unsafe,
                abi,
                params: params.iter().map(|t| self.compress_type(t)).collect(),
                return_type: return_type.as_ref().map(|t| self.compress_type(t)),
            },
            Type::GenericParam(_) => (**ty).clone(),
            Type::Subst(_) => unreachable!(),
        };

        self.alloc_subst(demangled);

        Arc::new(compressed)
    }

    fn alloc_subst(&mut self, key: String) {
        if !self.dict.contains_key(&key) {
            let subst = Subst(self.subst_counter);
            self.subst_counter += 1;
            self.dict.insert(key, subst);
        }
    }
}

#[cfg(test)]
impl CompressAlt {
    pub fn to_debug_dictionary(&self) -> DebugDictionary {
        DebugDictionary::new(
            self.dict
                .iter()
                .map(|(demangled, &subst)| (subst, demangled.clone()))
                .collect(),
        )
    }
}
