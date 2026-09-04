use crate::token::{Paren, Token};
use anyhow::{Result, bail};

pub fn parse_tokens(tokens: &[Token]) -> Result<f64> {
    if tokens.is_empty() {
        bail!("expected an expression");
    }
    let mut i = 0;
    let value = parse_expr(tokens, &mut i)?;
    if i != tokens.len() {
        bail!("expected an operator");
    }
    Ok(value)
}

fn parse_expr(tokens: &[Token], i: &mut usize) -> Result<f64> {
    let mut value = parse_term(tokens, i)?;
    loop {
        match tokens.get(*i) {
            Some(Token::Series) => {
                *i += 1;
                value += parse_term(tokens, i)?;
            }
            Some(Token::Parallel) => {
                *i += 1;
                value = parallel(value, parse_term(tokens, i)?);
            }
            Some(Token::Paren(Paren::R)) | None => return Ok(value),
            Some(_) => bail!("expected an operator"),
        }
    }
}

fn parse_term(tokens: &[Token], i: &mut usize) -> Result<f64> {
    match tokens.get(*i) {
        Some(&Token::Number(n)) => {
            *i += 1;
            Ok(n)
        }
        Some(Token::Paren(Paren::L)) => {
            *i += 1;
            let value = parse_expr(tokens, i)?;
            if matches!(tokens.get(*i), Some(Token::Paren(Paren::R))) {
                *i += 1;
                Ok(value)
            } else {
                bail!("expected ')'")
            }
        }
        _ => bail!("expected a value"),
    }
}

fn parallel(a: f64, b: f64) -> f64 {
    if a == 0.0 || b == 0.0 {
        0.0
    } else {
        1.0 / (1.0 / a + 1.0 / b)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::token::tokenize;

    fn parse(s: &str) -> Result<f64> {
        parse_tokens(&tokenize(s)?)
    }

    fn assert_res(actual: Result<f64>, expected: f64) {
        match actual {
            Ok(v) => assert!((v - expected).abs() < 1e-9, "expected {expected}, got {v}"),
            Err(e) => panic!("parse error: {e}"),
        }
    }

    #[test]
    fn test_single_value() {
        assert_res(parse("4.7k"), 4700.0);
        assert_res(parse("-4"), -4.0);
        assert_res(parse("100"), 100.0);
    }

    #[test]
    fn test_series_and_parallel() {
        assert_res(parse("4 + 3"), 7.0);
        assert_res(parse("4 // 3"), 4.0 * 3.0 / 7.0);
        assert_res(
            parse("4 // 3 // 2"),
            1.0 / (1.0 / 4.0 + 1.0 / 3.0 + 1.0 / 2.0),
        );
        assert_res(parse("4 + 3 + 2"), 9.0);
    }

    #[test]
    fn test_left_to_right_equal_precedence() {
        assert_res(parse("4 // 3 + 5"), 4.0 * 3.0 / 7.0 + 5.0);
        assert_res(parse("4 + 3 // 2"), (4.0 + 3.0) * 2.0 / 9.0);
        assert_res(
            parse("4 // 3 // 6"),
            1.0 / (1.0 / 4.0 + 1.0 / 3.0 + 1.0 / 6.0),
        );
    }

    #[test]
    fn test_parens() {
        assert_res(parse("(4 // 3) + 5"), 4.0 * 3.0 / 7.0 + 5.0);
        assert_res(parse("4 // (3 + 5)"), 4.0 * 8.0 / 12.0);
        assert_res(parse("((2))"), 2.0);
        assert_res(parse("(4 // 3) // 2"), 12.0 / 13.0);
    }

    #[test]
    fn test_parallel_zero() {
        assert_res(parse("0 // 4"), 0.0);
        assert_res(parse("4 // 0"), 0.0);
        assert_res(parse("0 // 0"), 0.0);
    }

    #[test]
    fn test_rejects() {
        for bad in [
            "", "4 // 3 +", "// 4", "4 //", "4 5", "3 - 4", "(3", "3)", "4 + + 3", "()", "(3 ) 4",
            "4 // (3", "4) + 3",
        ] {
            assert!(parse(bad).is_err(), "should reject {bad}");
        }
    }
}
