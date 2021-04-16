use std::fmt;

pub struct RadixFmt {
    radix: u8,
    value: u64,
}

const DIGITS: &[u8] = b"0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ";

pub fn radix(radix: u8, value: u64) -> RadixFmt {
    RadixFmt { radix, value }
}

impl fmt::Display for RadixFmt {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let RadixFmt { radix, mut value } = *self;

        assert!((2..=62).contains(&radix));

        if value == 0 {
            write!(f, "0")?;
        }

        let mut text = Vec::with_capacity(8);
        let radix = radix as u64;

        while value > 0 {
            let digit = value % radix;
            value /= radix;
            text.push(DIGITS[digit as usize]);
        }

        text.reverse();

        write!(f, "{}", String::from_utf8(text).unwrap())
    }
}

pub fn ascii_digit_to_value(ascii_char: u8, radix: u8) -> Option<u64> {
    let value = match ascii_char {
        b'0'..=b'9' => ascii_char - b'0',
        b'a'..=b'z' => (ascii_char - b'a') + 10,
        b'A'..=b'Z' => (ascii_char - b'A') + 36,
        _ => return None,
    };

    if value < radix {
        Some(value as u64)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str;
    use std::u64;

    #[test]
    fn ascii_digit_to_value_cross_check() {
        for (i, &d) in DIGITS.iter().enumerate() {
            for radix in 0..DIGITS.len() {
                if i < radix {
                    assert_eq!(Some(i as u64), ascii_digit_to_value(d, radix as u8));
                } else {
                    assert_eq!(None, ascii_digit_to_value(d, radix as u8));
                }
            }
        }
    }

    #[test]
    fn radix_fmt_display_zero() {
        for base in 2..=62 {
            assert_eq!("0", &format!("{}", radix(base as u8, 0)));
        }
    }

    #[test]
    fn radix_fmt_display_max_digit() {
        for base in 2..=62 {
            let expected = &[DIGITS[base - 1]];
            let expected = str::from_utf8(expected).unwrap();
            let actual = format!("{}", radix(base as u8, base as u64 - 1));
            assert_eq!(expected, actual);
        }
    }

    quickcheck! {
        fn radix_fmt_vs_std(value: u64, base: u8) -> bool {
            if !(2..=36).contains(&base) {
                return true
            }

            let text = &format!("{}",radix(base, value));
            value == u64::from_str_radix(text, base as u32).unwrap()
        }
    }
}
