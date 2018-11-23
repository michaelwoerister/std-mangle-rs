use ast::*;

mod arbitrary;

quickcheck! {
    fn compressing_ast_does_not_crash(symbol: Symbol) -> bool {
        ::compress_ast(&symbol);
        true
    }
}

quickcheck! {
    fn parsing_mangled_symbol_yields_same_ast(symbol: Symbol) -> bool {
        let mangled = ::ast_to_mangled_symbol(&symbol);
        match ::mangled_symbol_to_ast(&mangled) {
            Ok(parsed) => {
                if symbol != parsed {
                    panic!("Re-parsed symbol differs from original.\n
                            expected: {:?}\n\
                            actual:   {:?}\n\
                            mangled:  {}\n",
                            symbol,
                            parsed,
                            mangled)
                } else {
                    true
                }
            }
            Err(e) => {
                panic!("{}", e)
            }
        }
    }
}

quickcheck! {
    fn parsing_compressed_symbol_yields_same_ast(symbol: Symbol) -> bool {
        let compressed = ::compress_ast(&symbol);
        let mangled = ::ast_to_mangled_symbol(&compressed);
        match ::mangled_symbol_to_ast(&mangled) {
            Ok(parsed) => {
                if parsed != compressed {
                    panic!("Re-parsed compressed symbol differs from original.\n
                            expected: {:?}\n\
                            actual:   {:?}\n\
                            mangled:  {}\n",
                            compressed,
                            parsed,
                            mangled)
                } else {
                    true
                }
            }
            Err(e) => {
                panic!("{}", e)
            }
        }
    }
}

quickcheck! {
    fn demangle_direct_verbose(symbol: Symbol) -> bool {
        let expected = ::ast_to_demangled_symbol(&symbol, true);
        let uncompressed_mangled = ::ast_to_mangled_symbol(&symbol);
        let (compressed_symbol, compression_state) = ::compress::compress_ext(&symbol);
        let compressed_mangled =::ast_to_mangled_symbol(&compressed_symbol);
        let (actual, demangling_dict) =  ::direct_demangle::Demangler::demangle_debug(compressed_mangled.as_bytes(), true);

        let actual = actual.unwrap();

        if actual != expected {
            demangling_dict.print_comparison(&compression_state.to_debug_dictionary());

            panic!("expected:     {}\n\
                    actual:       {}\n\
                    compressed:   {}\n\
                    uncompressed: {}\n",
                    expected,
                    actual,
                    compressed_mangled,
                    uncompressed_mangled)
        } else {
            true
        }
    }
}

quickcheck! {
    fn demangle_direct(symbol: Symbol) -> bool {
        let expected = ::ast_to_demangled_symbol(&symbol, false);
        let uncompressed_mangled = ::ast_to_mangled_symbol(&symbol);
        let (compressed_symbol, compression_state) = ::compress::compress_ext(&symbol);
        let compressed_mangled =::ast_to_mangled_symbol(&compressed_symbol);
        let (actual, demangling_dict) =  ::direct_demangle::Demangler::demangle_debug(compressed_mangled.as_bytes(), false);

        let actual = actual.unwrap();

        if actual != expected {
            demangling_dict.print_comparison(&compression_state.to_debug_dictionary());

            panic!("expected:     {}\n\
                    actual:       {}\n\
                    compressed:   {}\n\
                    uncompressed: {}\n",
                    expected,
                    actual,
                    compressed_mangled,
                    uncompressed_mangled)
        } else {
            true
        }
    }
}

quickcheck! {
    fn compress_decompress_leaves_ast_unchanged(symbol: Symbol) -> bool {
        let (compressed, compression_state) = ::compress::compress_ext(&symbol);
        let (decompressed, decompression_state) = ::decompress::decompress_ext(&compressed);

        if symbol != decompressed {
            let compression_dict = compression_state.to_debug_dictionary();
            let decompression_dict = decompression_state.to_debug_dictionary();

            decompression_dict.print_comparison(&compression_dict);

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

quickcheck! {
    fn compress_ref_decompress_leaves_ast_unchanged(symbol: Symbol) -> bool {
        let (compressed, compression_state) = ::compress_ref::compress_ext(&symbol);
        let (decompressed, decompression_state) = ::decompress::decompress_ext(&compressed);

        if symbol != decompressed {
            let compression_dict = compression_state.to_debug_dictionary();
            let decompression_dict = decompression_state.to_debug_dictionary();

            decompression_dict.print_comparison(&compression_dict);

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
