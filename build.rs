use std::env;
use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::path::Path;

fn main() {
    let out_dir = env::var("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("generated_tests.rs");
    let mut output = File::create(&dest_path).unwrap();

    let test_case_definitions_path = Path::new("src").join("demangling_test_data.txt");

    println!(
        "cargo:rerun-if-changed={}",
        test_case_definitions_path.to_string_lossy()
    );

    let test_case_definitions = BufReader::new(File::open(test_case_definitions_path).unwrap());

    let lines: Vec<_> = test_case_definitions.lines().map(|l| l.unwrap()).collect();

    for (title_line, spec_line) in lines.iter().zip(lines.iter().skip(1)) {
        if title_line.starts_with('#') && spec_line.starts_with("_R") {
            emit_test_case(spec_line, title_line, &mut output);
        }
    }
}

fn emit_test_case(spec_line: &str, title_line: &str, output: &mut impl Write) {
    let end_of_mangled_name = spec_line.find(' ').unwrap();
    let mangled = &spec_line[..end_of_mangled_name];
    let demangled = spec_line[end_of_mangled_name + 1..].trim();

    let title = title_line[1..]
        .trim()
        .replace(" ", "_")
        .replace("/", "_")
        .replace(",", "_")
        .replace("-", "_");

    writeln!(output, "#[test] #[allow(non_snake_case)] fn {}() {{", title).unwrap();
    writeln!(output, "  let demangled_expected = r#\"{}\"#;", demangled).unwrap();
    writeln!(
        output,
        "  let ast = ::mangled_symbol_to_ast(r#\"{}\"#).unwrap();",
        mangled
    )
    .unwrap();
    writeln!(
        output,
        "  let demangled_actual = ::ast_to_demangled_symbol(&ast);"
    )
    .unwrap();
    writeln!(
        output,
        "  assert_eq!(demangled_expected, demangled_actual);"
    )
    .unwrap();
    writeln!(output, "}}").unwrap();
}
