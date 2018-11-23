use std::env;
use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::path::Path;

fn main() {
    let out_dir = env::var("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("generated_tests.rs");
    let mut output = File::create(&dest_path).unwrap();

    let test_case_definitions_path = Path::new("src").join("demangling_test_data.txt");

    let test_case_definitions = BufReader::new(File::open(test_case_definitions_path).unwrap());

    let lines: Vec<_> = test_case_definitions.lines().map(|l| l.unwrap()).collect();

    for i in 1 .. lines.len() - 1 {
        if lines[i].starts_with("_R") && lines[i-1].starts_with("#") {
            let title_line = &lines[i-1];
            let spec_line = &lines[i];

            emit_test_case_ast(spec_line, title_line, &mut output, true);
            emit_test_case_direct(spec_line, title_line, &mut output, true);

            // build non-verbose pseudoline
            assert!(!lines[i+1].trim().is_empty() && !lines[i+1].starts_with("#"),
                    "non-verbose test case missing for test at line {}",
                    i-1);

            let non_verbose_spec_line = &format!("{} {}",
                extract_mangled_name_from_spec_line(spec_line),
                lines[i+1].trim());

            emit_test_case_ast(non_verbose_spec_line, title_line, &mut output, false);
            emit_test_case_direct(non_verbose_spec_line, title_line, &mut output, false);
        }
    }
}

fn extract_mangled_name_from_spec_line(spec_line: &str) -> &str {
    assert!(spec_line.starts_with("_R"));
    let end_of_mangled_name = spec_line.find(' ').unwrap();
    &spec_line[..end_of_mangled_name]
}

fn emit_test_case_ast(spec_line: &str, title_line: &str, output: &mut impl Write, verbose: bool) {
    if spec_line.starts_with("_R") && title_line.starts_with("#") {
        let end_of_mangled_name = spec_line.find(' ').unwrap();
        let mangled = &spec_line[..end_of_mangled_name];
        let demangled = spec_line[end_of_mangled_name + 1..].trim();

        let mut title = title_line[1..]
            .trim()
            .replace(" ", "_")
            .replace("/", "_")
            .replace(",", "_")
            .replace("-", "_") + "_ast";

        if verbose {
            title += "_verbose";
        }

        writeln!(output, "#[test] #[allow(non_snake_case)] fn {}() {{", title).unwrap();
        writeln!(output, "  let demangled_expected = r#\"{}\"#;", demangled).unwrap();
        writeln!(
            output,
            "  let ast = ::mangled_symbol_to_ast(r#\"{}\"#).unwrap();",
            mangled
        ).unwrap();
        writeln!(output, "  let decompressed = ::decompress_ast(&ast);").unwrap();
        writeln!(
            output,
            "  let demangled_actual = ::ast_to_demangled_symbol(&decompressed, {});",
            verbose
        ).unwrap();
        writeln!(
            output,
            "  assert_eq!(demangled_expected, demangled_actual);"
        ).unwrap();
        writeln!(output, "}}").unwrap();
    }
}

fn emit_test_case_direct(spec_line: &str, title_line: &str, output: &mut impl Write, verbose: bool) {
    if spec_line.starts_with("_R") && title_line.starts_with("#") {
        let end_of_mangled_name = spec_line.find(' ').unwrap();
        let mangled = &spec_line[..end_of_mangled_name];
        let demangled = spec_line[end_of_mangled_name + 1..].trim();

        let mut title = title_line[1..]
            .trim()
            .replace(" ", "_")
            .replace("/", "_")
            .replace(",", "_")
            .replace("-", "_") + "_direct";

        if verbose {
            title += "_verbose";
        }

        writeln!(output, "#[test] #[allow(non_snake_case)] fn {}() {{", title).unwrap();
        writeln!(output, "  let demangled_expected = r#\"{}\"#;", demangled).unwrap();
        writeln!(
            output,
            "  let demangled_expected = Ok(demangled_expected.to_string());"
        ).unwrap();
        writeln!(
            output,
            "  let demangled_actual = ::demangle_symbol(\"{}\", {});",
            mangled,
            verbose
        ).unwrap();
        writeln!(
            output,
            "  assert_eq!(demangled_expected, demangled_actual);"
        ).unwrap();
        writeln!(output, "}}").unwrap();
    }
}
