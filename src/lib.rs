#![cfg(any(target_os = "windows", target_os = "linux"))]
#![allow(internal_features)]
#![feature(fmt_internals)]

mod column_line;
mod constants;
mod matcher;
mod platform_const;

use std::any::Any;

use anyhow::{Ok, Result};
use column_line::*;
use constants::*;
use derive_more::derive::Display;
pub use matcher::*;
use regex::Regex;

#[derive(Clone)]
pub struct Token {
    pub val: String,
    pub line: usize,
    pub column: usize,
}

impl ::std::fmt::Debug for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Token {{value: {:#?}, line: {:#?}, column: {:#?}}}",
            self.val, self.line, self.column
        )
    }
}

impl ::std::fmt::Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        <Self as ::std::fmt::Debug>::fmt(self, f)
    }
}

pub struct Tokenizer {
    matchers: Vec<Matcher>,
    str_iter: String,
}

impl Tokenizer {
    pub fn new(s: impl ToString) -> Self {
        Self {
            matchers: Default::default(),
            str_iter: s.to_string(),
        }
    }

    pub fn add_str_pat<T: ToString>(&mut self, src: T) {
        self.matchers.push(src.to_string().into());
    }

    pub fn add_str_pattern_array<T: ToString, const N: usize>(&mut self, src: [T; N]) {
        for s in src {
            self.add_str_pat(s);
        }
    }

    pub fn add_str_pattern_vec<T: ToString>(&mut self, src: Vec<T>) {
        for s in src {
            self.add_str_pat(s);
        }
    }

    pub fn add_pattern_array<T: MatcherTrait + Any, const N: usize>(&mut self, src: [T; N]) {
        for s in src {
            self.add_pat(s);
        }
    }

    pub fn add_pattern_vec<T: MatcherTrait + Any>(&mut self, src: Vec<T>) {
        for s in src {
            self.add_pat(s);
        }
    }

    pub fn add_pat<T: MatcherTrait + Any>(&mut self, src: T) {
        self.matchers.push(src.into());
    }

    pub fn add_regex_pat(&mut self, src: impl ToString) -> Result<()> {
        self.matchers
            .push(Regex::new(src.to_string().as_str())?.into());
        Ok(())
    }

    pub fn add_regex_pattern_array<T: ToString, const N: usize>(
        &mut self,
        src: [T; N],
    ) -> Result<()> {
        for s in src {
            self.add_regex_pat(s)?;
        }
        Ok(())
    }

    pub fn add_regex_pattern_vec<T: ToString>(&mut self, src: Vec<T>) -> Result<()> {
        for s in src {
            self.add_regex_pat(s)?;
        }
        Ok(())
    }

    pub fn add_ws_pat(&mut self) {
        self.matchers.push(WHITE_SPACE_REGEX.into());
    }

    pub(crate) fn add_common_pat(&mut self, src: Matcher) {
        self.matchers.push(src);
    }

    pub fn start(&mut self) -> Result<Vec<Token>> {
        let mut res = Vec::with_capacity(self.matchers.len());
        let lookup = LineColLookup::new(&self.str_iter);
        let mut current_str = self.str_iter.clone();
        let mut current_index = 0;
        loop {
            let mut matched = false;
            for reg in &self.matchers {
                if let Some(s) = reg.get(&current_str) {
                    current_str = current_str[s.len()..].to_owned();
                    let (line, column) = lookup.get(current_index);
                    current_index += s.len();
                    res.push(Token {
                        val: s,
                        line,
                        column,
                    });
                    matched = true;
                    break;
                } else {
                    continue;
                }
            }
            if !matched {
                return Err(TokenizerError::AllMatchersMatchNothing.into());
            }
            if current_str.len() == 0 {
                break;
            }
        }
        Ok(res)
    }
}

#[derive(Debug, Display, thiserror::Error)]
pub enum TokenizerError {
    AllMatchersMatchNothing,
}

#[inline]
pub fn build_tokenizer<TSrc: ToString>(val: Vec<Matcher>, src: TSrc) -> Tokenizer {
    let mut tokenizer = Tokenizer::new(src);
    for v in val {
        tokenizer.add_common_pat(v);
    }
    tokenizer
}

#[inline]
pub fn to_tokens<TSrc: ToString>(val: Vec<Matcher>, src: TSrc) -> Result<Vec<Token>> {
    let mut tokenizer = build_tokenizer(val, src);
    tokenizer.start()
}

#[inline]
pub fn to_tokens_without_ws<TSrc: ToString>(val: Vec<Matcher>, src: TSrc) -> Result<Vec<Token>> {
    let mut tokenizer = build_tokenizer(val, src);
    Ok(filter_white_spaces(tokenizer.start()?))
}

#[inline]
pub fn filter_white_spaces(val: Vec<Token>) -> Vec<Token> {
    val.iter()
        .filter(|x| !WHITE_SPACE_REGEX.is_match(&x.val))
        .map(|x| x.clone())
        .collect()
}

#[cfg(test)]
mod tests {
    use regex::Regex;

    use super::*;

    #[test]
    #[allow(unused_must_use)]
    fn t1() {
        let src = "
class Test {
}";
        let tokens = filter_white_spaces(
            to_tokens(
                vec![
                    "class".into(),
                    WHITE_SPACE_REGEX.clone().into(),
                    Regex::new(r"\A[.[^\{\s]]+").unwrap().into(),
                    "{".into(),
                    "}".into(),
                ],
                src,
            )
            .unwrap(),
        );
        dbg!(tokens);
    }
    #[test]
    #[allow(unused_must_use)]
    fn test_string_parse() {
        let src = r#""sudsier\" asdf \"""#;
        let mut tokenizer = Tokenizer::new(src);
        tokenizer.add_ws_pat();
        tokenizer.add_regex_pat(r#"\A"[[.[^"]]\\"]*""#).unwrap();
        dbg!(tokenizer.start());
    }
}
