use punycode;

pub fn decode_punycode_ident(ident_bytes: &[u8]) -> Result<String, String> {
    if ident_bytes.iter().any(|b| !b.is_ascii()) {
        return Err(format!(
            "Ident '{}' unexpectedly contains non-ascii characters.",
            String::from_utf8_lossy(ident_bytes)
        ));
    }

    let mut ident_bytes = ident_bytes.to_owned();

    if let Some(index) = ident_bytes.iter().rposition(|&c| c == b'_') {
        ident_bytes[index] = b'-';
        remap_punycode_charset_AJ_to_09(&mut ident_bytes[index..]);
    } else {
        remap_punycode_charset_AJ_to_09(&mut ident_bytes[..]);
    };

    let ident_str = String::from_utf8(ident_bytes).unwrap();

    match punycode::decode_to_string(&ident_str) {
        Some(s) => Ok(s),
        None => {
            return Err(format!(
                "Could not decode punycode-encoded ident '{}'.",
                ident_str
            ));
        }
    }
}

#[allow(non_snake_case)]
fn remap_punycode_charset_AJ_to_09(punycode_suffix: &mut [u8]) {
    for c in punycode_suffix {
        if *c >= b'A' && *c <= b'J' {
            *c = (*c - b'A') + b'0';
        }
    }
}
