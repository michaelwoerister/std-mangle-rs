use std::fmt::{Display, Write};

pub fn expected<T>(
    expected_chars: &str,
    found_char: u8,
    verb: &str,
    noun: &str,
) -> Result<T, String> {
    let expected_chars = expected_chars.chars().collect::<Vec<_>>();

    assert!(!expected_chars.is_empty());

    let mut message = "Expected ".to_string();

    if expected_chars.len() == 1 {
        write!(message, "{}", char_to_str(expected_chars[0])).unwrap();
    } else if expected_chars.len() == 2 {
        write!(
            message,
            "{} or {}",
            char_to_str(expected_chars[0]),
            char_to_str(expected_chars[1])
        )
        .unwrap();
    } else {
        for i in 0..expected_chars.len() - 1 {
            write!(message, "{}, ", char_to_str(expected_chars[i])).unwrap();
        }

        write!(
            message,
            "or {}",
            char_to_str(expected_chars[expected_chars.len() - 1])
        )
        .unwrap();
    }

    write!(
        message,
        "; found {} instead; while {} {}",
        char_to_str(found_char as char),
        verb,
        noun
    )
    .unwrap();

    Err(message)
}

fn char_to_str(c: char) -> String {
    match c {
        '#' => "digit".to_string(),
        c => format!("'{}'", c),
    }
}

pub fn version_mismatch<T, V>(actual_version: V, supported_versions: V) -> Result<T, String>
where
    V: Display,
{
    Err(format!(
        "Symbol uses encoding version {} but the highest supported version is {}.",
        actual_version, supported_versions
    ))
}
