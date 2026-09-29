use anyhow::{Result, anyhow, bail};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Paren {
    L,
    R,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Token {
    Number(f64),
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
            '0'..='9' | '-' => tokens.push(Token::Number(scan_number(input, &mut chars)?)),
            _ => bail!("unknown token {c}"),
        }
    }
    Ok(tokens)
}

fn scan_number(input: &str, chars: &mut std::iter::Peekable<std::str::CharIndices>) -> Result<f64> {
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
    let mantissa: f64 = mantissa.parse().map_err(|_| anyhow!("malformed number"))?;
    let value = mantissa * scale;
    if !value.is_finite() {
        bail!("number out of range");
    }
    Ok(value)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_tokenize() {
        let s = "4 // 3 + 5";
        assert_eq!(
            tokenize(s).as_deref().unwrap(),
            [
                Token::Number(4.0),
                Token::Parallel,
                Token::Number(3.0),
                Token::Series,
                Token::Number(5.0)
            ]
            .as_slice()
        )
    }

    #[test]
    fn test_glue() {
        assert_eq!(
            tokenize("4//3").as_deref().unwrap(),
            [Token::Number(4.0), Token::Parallel, Token::Number(3.0)].as_slice()
        );
        assert_eq!(
            tokenize("(3+5)").as_deref().unwrap(),
            [
                Token::Paren(Paren::L),
                Token::Number(3.0),
                Token::Series,
                Token::Number(5.0),
                Token::Paren(Paren::R)
            ]
            .as_slice()
        );
        assert_eq!(
            tokenize("4.7k+3").as_deref().unwrap(),
            [Token::Number(4700.0), Token::Series, Token::Number(3.0)].as_slice()
        );
    }

    #[test]
    fn test_suffixes() {
        assert_eq!(
            tokenize("10k").as_deref().unwrap(),
            [Token::Number(10_000.0)].as_slice()
        );
        assert_eq!(
            tokenize("1M").as_deref().unwrap(),
            [Token::Number(1_000_000.0)].as_slice()
        );
        assert_eq!(
            tokenize("4.7m").as_deref().unwrap(),
            [Token::Number(0.0047)].as_slice()
        );
        assert_eq!(
            tokenize("100").as_deref().unwrap(),
            [Token::Number(100.0)].as_slice()
        );
    }

    #[test]
    fn test_negative() {
        assert_eq!(
            tokenize("3-4").as_deref().unwrap(),
            [Token::Number(3.0), Token::Number(-4.0)].as_slice()
        );
        assert_eq!(
            tokenize("-4k").as_deref().unwrap(),
            [Token::Number(-4000.0)].as_slice()
        );
    }

    #[test]
    fn test_rejects() {
        for bad in ["4..", "..", "4-", "--4", "foo", "/", "4 / 3", "-.5", "9e3"] {
            assert!(tokenize(bad).is_err(), "should reject {bad}");
        }
    }
}
