use std::collections::HashMap;

use crate::complex::Complex;
use crate::token::{Paren, Token};

use anyhow::{Result, anyhow, bail};

fn parse_tokens<'a>(tokens: &[Token<'a>], vars: &HashMap<String, Complex>) -> Result<Complex> {
    if tokens.is_empty() {
        bail!("expected an expression");
    }
    let mut i = 0;
    let value = parse_expr(tokens, &mut i, vars)?;
    if i != tokens.len() {
        bail!("expected an operator");
    }
    Ok(value)
}

fn parse_expr<'a>(
    tokens: &[Token<'a>],
    i: &mut usize,
    vars: &HashMap<String, Complex>,
) -> Result<Complex> {
    let mut value = parse_term(tokens, i, vars)?;
    loop {
        match tokens.get(*i) {
            Some(Token::Series) => {
                *i += 1;
                value += parse_term(tokens, i, vars)?;
            }
            Some(Token::Parallel) => {
                *i += 1;
                value = parallel(value, parse_term(tokens, i, vars)?);
            }
            Some(Token::Paren(Paren::R)) | None => return Ok(value),
            Some(_) => bail!("expected an operator"),
        }
    }
}

fn parse_term<'a>(
    tokens: &[Token<'a>],
    i: &mut usize,
    vars: &HashMap<String, Complex>,
) -> Result<Complex> {
    match tokens.get(*i) {
        Some(&Token::Number(n)) => {
            *i += 1;
            Ok(n)
        }
        Some(&Token::Ident(name)) => {
            *i += 1;
            vars.get(name)
                .copied()
                .ok_or_else(|| anyhow!("undefined variable '{name}'"))
        }
        Some(Token::Paren(Paren::L)) => {
            *i += 1;
            let value = parse_expr(tokens, i, vars)?;
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

fn parallel(a: Complex, b: Complex) -> Complex {
    if a.is_zero() || b.is_zero() {
        Complex::ZERO
    } else {
        Complex::ONE / (a.recip() + b.recip())
    }
}

/// Tokenizes and parses `line`, reporting a top-level `name = expr` assignment
/// alongside the evaluated value. Nothing is stored here.
fn eval_with<'a>(
    line: &'a str,
    vars: &HashMap<String, Complex>,
) -> Result<(Option<&'a str>, Complex)> {
    let tokens = crate::token::tokenize(line)?;
    if let [Token::Ident(name), Token::Assign, rest @ ..] = tokens.as_slice() {
        Ok((Some(name), parse_tokens(rest, vars)?))
    } else {
        Ok((None, parse_tokens(&tokens, vars)?))
    }
}

/// Evaluates `line`, storing a top-level assignment into `vars` when provided.
/// Without `vars`, an assignment still evaluates to its right-hand side but is
/// not remembered.
///
/// The result is always stored under `ans` and `_` so later lines can reuse it.
pub fn eval(line: &str, vars: Option<&mut HashMap<String, Complex>>) -> Result<Complex> {
    match vars {
        Some(env) => {
            let (name, value) = eval_with(line, env)?;
            if let Some(name) = name {
                env.insert(name.to_string(), value);
            }
            env.insert("ans".to_string(), value);
            env.insert("_".to_string(), value);
            Ok(value)
        }
        None => {
            let (_, value) = eval_with(line, &HashMap::new())?;
            Ok(value)
        }
    }
}

/// Evaluates `line` against `vars` without storing an assignment. Intended for
/// the REPL answer preview.
pub fn eval_readonly(line: &str, vars: &HashMap<String, Complex>) -> Result<Complex> {
    eval_with(line, vars).map(|(_, value)| value)
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::token::tokenize;

    fn parse(s: &str) -> Result<Complex> {
        parse_tokens(&tokenize(s)?, &HashMap::new())
    }

    fn assert_res(actual: Result<Complex>, expected: impl Into<Complex>) {
        let expected = expected.into();
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
    fn test_imaginary() {
        // pure imaginary literals flow through both operators
        assert_res(parse("3j"), Complex::new(0.0, 3.0));
        assert_res(parse("i"), Complex::new(0.0, 1.0));
        assert_res(parse("3j + 4"), Complex::new(4.0, 3.0));
        // parallel of 4 and 3j is 1/(1/4 + 1/(3j))
        assert_res(
            parse("4 // 3j"),
            Complex::ONE / (Complex::from(4.0).recip() + Complex::new(0.0, 3.0).recip()),
        );
    }

    #[test]
    fn test_parallel_zero() {
        assert_res(parse("0 // 4"), 0.0);
        assert_res(parse("4 // 0"), 0.0);
        assert_res(parse("0 // 0"), 0.0);
    }

    #[test]
    fn test_variables() {
        let mut vars = HashMap::new();
        vars.insert("r1".to_string(), Complex::from(4700.0));
        vars.insert("r2".to_string(), Complex::new(0.0, 3000.0));

        assert_res(parse_tokens(&tokenize("r1 + 3k").unwrap(), &vars), 7700.0);
        assert_res(
            parse_tokens(&tokenize("r1 // r2").unwrap(), &vars),
            Complex::ONE / (Complex::from(4700.0).recip() + Complex::new(0.0, 3000.0).recip()),
        );
    }

    #[test]
    fn test_undefined_variable() {
        assert!(parse("nope + 1").is_err());
    }

    #[test]
    fn test_assignment() {
        let mut vars = HashMap::new();
        assert_res(eval("r1 = 4.7k", Some(&mut vars)), 4700.0);
        assert_eq!(vars.get("r1"), Some(&Complex::from(4700.0)));
        assert_res(eval("r1 // 3k", Some(&mut vars)), 1831.1688311688315);

        // without a map, an assignment evaluates but is not stored
        assert_res(eval("x = 4", None), 4.0);
    }

    #[test]
    fn test_last_answer_aliases() {
        let mut vars = HashMap::new();
        assert_res(eval("4 + 3", Some(&mut vars)), 7.0);
        assert_eq!(vars.get("ans"), Some(&Complex::from(7.0)));
        assert_eq!(vars.get("_"), Some(&Complex::from(7.0)));

        // the aliases are available on later lines
        assert_res(eval("ans + 1", Some(&mut vars)), 8.0);
        assert_eq!(vars.get("_"), Some(&Complex::from(8.0)));
        assert_res(eval("_ + 1", Some(&mut vars)), 9.0);

        // an assignment also updates the aliases
        assert_res(eval("r1 = 100", Some(&mut vars)), 100.0);
        assert_eq!(vars.get("ans"), Some(&Complex::from(100.0)));
        assert_eq!(vars.get("_"), Some(&Complex::from(100.0)));
    }

    #[test]
    fn test_assignment_rejects() {
        let mut vars = HashMap::new();
        for bad in ["x =", "= 4", "x = y = 4", "(x = 4)"] {
            assert!(eval(bad, Some(&mut vars)).is_err(), "should reject {bad}");
        }
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
