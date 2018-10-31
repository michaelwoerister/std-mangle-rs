use std::env;
use std::fs::File;
use std::path::Path;
use std::io::{Write, BufReader, BufRead};


fn main() {
    let out_dir = env::var("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("generated_tests.rs");
    let mut output = File::create(&dest_path).unwrap();

    let test_case_definitions_path =
        Path::new("src").join("demangling_test_data.txt");

    let test_case_definitions = BufReader::new(File::open(test_case_definitions_path).unwrap());


    let mut prev_line = String::new();

    for line in test_case_definitions.lines().map(|l| l.unwrap()) {
        if line.starts_with("_R") && prev_line.starts_with("#") {

            let end_of_mangled_name = line.find(' ').unwrap();
            let mangled = &line[..end_of_mangled_name];
            let demangled = line[end_of_mangled_name + 1 ..].trim();

            let title = prev_line[1 .. ].trim().replace(" ", "_")
                                               .replace("/", "_")
                                               .replace(",", "_")
                                               .replace("-", "_");

            writeln!(output, "#[test] #[allow(non_snake_case)] fn {}() {{", title);
            writeln!(output, "  let demangled_expected = \"{}\";", demangled);
            writeln!(output, "  let ast = ::parse::Parser::parse(b\"{}\").unwrap();", mangled);
            writeln!(output, "  let decompressed = ::decompress::Decompress::decompress(&ast);");
            writeln!(output, "  let mut demangled_actual = String::new();");
            writeln!(output, "  decompressed.pretty_print(&mut demangled_actual);");
            writeln!(output, "  assert_eq!(demangled_expected, demangled_actual);");
            writeln!(output, "}}");
        }

        prev_line = line;
    }
}
