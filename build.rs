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

    let mut prev_line = String::new();

    for line in test_case_definitions.lines().map(|l| l.unwrap()) {
        if line.starts_with("_R") && prev_line.starts_with("#") {
            emit_test_case_ast(&line, &prev_line, &mut output);
            emit_test_case_direct(&line, &prev_line, &mut output);
        }

        prev_line = line;
    }
}

fn emit_test_case_ast(spec_line: &str, title_line: &str, output: &mut impl Write) {
    if spec_line.starts_with("_R") && title_line.starts_with("#") {
        let end_of_mangled_name = spec_line.find(' ').unwrap();
        let mangled = &spec_line[..end_of_mangled_name];
        let demangled = spec_line[end_of_mangled_name + 1..].trim();

        let title = title_line[1..]
            .trim()
            .replace(" ", "_")
            .replace("/", "_")
            .replace(",", "_")
            .replace("-", "_") + "_ast";

        writeln!(output, "#[test] #[allow(non_snake_case)] fn {}() {{", title).unwrap();
        writeln!(output, "  let demangled_expected = r#\"{}\"#;", demangled).unwrap();
        writeln!(
            output,
            "  let ast = ::parse::Parser::parse(br#\"{}\"#).unwrap();",
            mangled
        ).unwrap();
        writeln!(
            output,
            "  let decompressed = ::decompress::Decompress::decompress(&ast);"
        ).unwrap();
        writeln!(output, "  let mut demangled_actual = String::new();").unwrap();
        writeln!(
            output,
            "  decompressed.pretty_print(&mut demangled_actual);"
        ).unwrap();
        writeln!(
            output,
            "  assert_eq!(demangled_expected, demangled_actual);"
        ).unwrap();
        writeln!(output, "}}").unwrap();
    }
}

fn emit_test_case_direct(spec_line: &str, title_line: &str, output: &mut impl Write) {
    if spec_line.starts_with("_R") && title_line.starts_with("#") {
        let end_of_mangled_name = spec_line.find(' ').unwrap();
        let mangled = &spec_line[..end_of_mangled_name];
        let demangled = spec_line[end_of_mangled_name + 1..].trim();

        let title = title_line[1..]
            .trim()
            .replace(" ", "_")
            .replace("/", "_")
            .replace(",", "_")
            .replace("-", "_") + "_direct";

        writeln!(output, "#[test] #[allow(non_snake_case)] fn {}() {{", title).unwrap();
        writeln!(output, "  let demangled_expected = r#\"{}\"#;", demangled).unwrap();
        writeln!(
            output,
            "  let demangled_expected = Ok(demangled_expected.to_string());"
        ).unwrap();
        writeln!(
            output,
            "  let demangled_actual = ::direct_demangle::Demangler::demangle(b\"{}\");",
            mangled
        ).unwrap();
        writeln!(
            output,
            "  assert_eq!(demangled_expected, demangled_actual);"
        ).unwrap();
        writeln!(output, "}}").unwrap();
    }
}
