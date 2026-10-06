use anyhow::{Result, anyhow, bail};

use crate::complex::Complex;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Paren {
    L,
    R,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Token {
    Number(Complex),
    Parallel,
    Series,
    Paren(Paren),
}

pub fn tokenize(input: &str) -> Result<Vec<Token>> {
    let mut chars = input.char_indices().peekable();
    let mut tokens = Vec::new();
    while let Some(&(_, c)) = chars.peek() {
        match c {
            c if c.is_ascii_whitespace() => {
                chars.next();
            }
            '(' => {
                tokens.push(Token::Paren(Paren::L));
                chars.next();
            }
            ')' => {
                tokens.push(Token::Paren(Paren::R));
                chars.next();
            }
            '+' => {
                tokens.push(Token::Series);
                chars.next();
            }
            '/' => {
                chars.next();
                match chars.next() {
                    Some((_, '/')) => tokens.push(Token::Parallel),
                    _ => bail!("expected '//'"),
                }
            }
            'i' | 'j' => {
                chars.next();
                tokens.push(Token::Number(Complex::new(0.0, 1.0)));
            }
            '0'..='9' | '-' => tokens.push(Token::Number(scan_number(input, &mut chars)?)),
            _ => bail!("unknown token {c}"),
        }
    }
    Ok(tokens)
}

fn scan_number(
    input: &str,
    chars: &mut std::iter::Peekable<std::str::CharIndices>,
) -> Result<Complex> {
    let start = match chars.peek() {
        Some(&(i, '-')) => {
            chars.next();
            i
        }
        Some(&(i, _)) => i,
        None => bail!("expected number"),
    };
    if matches!(chars.peek(), Some(&(_, '.'))) {
        bail!("expected number");
    }
    let mut digits = 0usize;
    let mut dots = 0usize;
    let mut end = start;
    while let Some(&(idx, c)) = chars.peek() {
        match c {
            '0'..='9' => digits += 1,
            '.' => dots += 1,
            _ => break,
        }
        end = idx + c.len_utf8();
        chars.next();
    }
    if digits == 0 {
        bail!("expected number");
    }
    if dots > 1 {
        bail!("malformed number");
    }
    let mantissa = &input[start..end];
    let mut scale = 1.0;
    if let Some(&(idx, c)) = chars.peek() {
        match c {
            'k' | 'K' => {
                scale = 1e3;
                _ = idx;
                chars.next();
            }
            'M' => {
                scale = 1e6;
                _ = idx;
                chars.next();
            }
            'm' => {
                scale = 1e-3;
                _ = idx;
                chars.next();
            }
            _ => {}
        }
    }
    // `i`/`j` marks the value as purely imaginary. It follows the scale
    // suffix, so `4.7kj` is 4700j while `4.7j` is 4.7j.
    let mut imaginary = false;
    if let Some(&(_, 'i' | 'j')) = chars.peek() {
        imaginary = true;
        chars.next();
    }
    let mantissa: f64 = mantissa.parse().map_err(|_| anyhow!("malformed number"))?;
    let value = mantissa * scale;
    if !value.is_finite() {
        bail!("number out of range");
    }
    // A leading `-` is part of the scanned mantissa, so the sign already
    // rides along on `value`; it just belongs to the imaginary part here.
    Ok(if imaginary {
        Complex::new(0.0, value)
    } else {
        Complex::new(value, 0.0)
    })
}

#[cfg(test)]
mod test {
    use super::*;

    fn num(re: f64) -> Token {
        Token::Number(re.into())
    }

    #[test]
    fn test_tokenize() {
        let s = "4 // 3 + 5";
        assert_eq!(
            tokenize(s).as_deref().unwrap(),
            [num(4.0), Token::Parallel, num(3.0), Token::Series, num(5.0)].as_slice()
        )
    }

    #[test]
    fn test_glue() {
        assert_eq!(
            tokenize("4//3").as_deref().unwrap(),
            [num(4.0), Token::Parallel, num(3.0)].as_slice()
        );
        assert_eq!(
            tokenize("(3+5)").as_deref().unwrap(),
            [
                Token::Paren(Paren::L),
                num(3.0),
                Token::Series,
                num(5.0),
                Token::Paren(Paren::R)
            ]
            .as_slice()
        );
        assert_eq!(
            tokenize("4.7k+3").as_deref().unwrap(),
            [num(4700.0), Token::Series, num(3.0)].as_slice()
        );
    }

    #[test]
    fn test_suffixes() {
        assert_eq!(
            tokenize("10k").as_deref().unwrap(),
            [num(10_000.0)].as_slice()
        );
        assert_eq!(
            tokenize("1M").as_deref().unwrap(),
            [num(1_000_000.0)].as_slice()
        );
        assert_eq!(
            tokenize("4.7m").as_deref().unwrap(),
            [num(0.0047)].as_slice()
        );
        assert_eq!(tokenize("100").as_deref().unwrap(), [num(100.0)].as_slice());
    }

    #[test]
    fn test_negative() {
        assert_eq!(
            tokenize("3-4").as_deref().unwrap(),
            [num(3.0), num(-4.0)].as_slice()
        );
        assert_eq!(
            tokenize("-4k").as_deref().unwrap(),
            [num(-4000.0)].as_slice()
        );
    }

    fn im(re: f64, im: f64) -> Token {
        Token::Number(Complex::new(re, im))
    }

    #[test]
    fn test_imaginary_suffix() {
        // `i` and `j` are interchangeable
        for s in ["3i", "3j"] {
            assert_eq!(
                tokenize(s).as_deref().unwrap(),
                [im(0.0, 3.0)].as_slice(),
                "{s}"
            );
        }
        for s in ["-3i", "-3j"] {
            assert_eq!(
                tokenize(s).as_deref().unwrap(),
                [im(0.0, -3.0)].as_slice(),
                "{s}"
            );
        }
        assert_eq!(
            tokenize("4.5i").as_deref().unwrap(),
            [im(0.0, 4.5)].as_slice()
        );
    }

    #[test]
    fn test_imaginary_bare_unit() {
        for s in ["i", "j"] {
            assert_eq!(
                tokenize(s).as_deref().unwrap(),
                [im(0.0, 1.0)].as_slice(),
                "{s}"
            );
        }
        // as a multiplier, and inside a series expression
        assert_eq!(
            tokenize("3j + 4").as_deref().unwrap(),
            [im(0.0, 3.0), Token::Series, num(4.0)].as_slice()
        );
    }

    #[test]
    fn test_imaginary_with_scale() {
        // the unit follows the scale suffix
        assert_eq!(
            tokenize("4.7kj").as_deref().unwrap(),
            [im(0.0, 4700.0)].as_slice()
        );
        assert_eq!(
            tokenize("2ki").as_deref().unwrap(),
            [im(0.0, 2000.0)].as_slice()
        );
        assert_eq!(
            tokenize("1.5Mj").as_deref().unwrap(),
            [im(0.0, 1_500_000.0)].as_slice()
        );
        assert_eq!(
            tokenize("10mi").as_deref().unwrap(),
            [im(0.0, 0.01)].as_slice()
        );
    }

    #[test]
    fn test_rejects() {
        for bad in ["4..", "..", "4-", "--4", "foo", "/", "4 / 3", "-.5", "9e3"] {
            assert!(tokenize(bad).is_err(), "should reject {bad}");
        }
    }
}
