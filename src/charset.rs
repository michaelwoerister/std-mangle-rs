use punycode;
use std::fmt::Write;
use std::str;

// TODO: document this
pub fn write_len_prefixed_ident(ident: &[&str], out: &mut String) -> Result<(), String> {
    let use_punycode = ident.iter().any(|x| !x.is_ascii());

    if use_punycode {
        let ident = ident.join("");

        assert_eq!(ident.find('-'), None);

        let mut encoded_ident = if let Some(encoded_ident) = punycode::encode_str(&ident) {
            encoded_ident.into_bytes()
        } else {
            return Err(format!(
                "The identifier '{}' cannot be encoded to punycode.",
                ident
            ));
        };

        assert!(encoded_ident.iter().all(u8::is_ascii));

        if let Some(index) = encoded_ident.iter().rposition(|&c| c == b'-') {
            encoded_ident[index] = b'_';
            remap_punycode_charset_09_to_AJ(&mut encoded_ident[index..]);
        } else {
            // The ident consisted entirely of non-ascii characters.
            remap_punycode_charset_09_to_AJ(&mut encoded_ident[..]);
        }

        let encoded_ident = String::from_utf8(encoded_ident).unwrap();

        write!(out, "{}{}u", encoded_ident.len(), encoded_ident).unwrap();
    } else {
        let len: usize = ident.iter().map(|component| component.len()).sum();

        write!(out, "{}", len).unwrap();

        for component in ident {
            out.push_str(component);
        }
    }

    Ok(())
}

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
fn remap_punycode_charset_09_to_AJ(punycode_suffix: &mut [u8]) {
    for c in punycode_suffix {
        if *c >= b'0' && *c <= b'9' {
            *c = (*c - b'0') + b'A';
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
