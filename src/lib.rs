#![cfg(any(target_os = "windows", target_os = "linux"))]
#![feature(fmt_internals)]

mod column_line;
mod constants;
mod matcher;
mod platform_const;

use std::any::Any;

use column_line::*;
use constants::{DEBUG, WHITE_SPACE_REGEX};
use derive_more::derive::{Display, Error};
pub use matcher::*;

type Res<T> = Result<T, TokenizerError>;

#[derive(Clone)]
pub struct Token {
    val: String,
    line: usize,
    column: usize,
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

    pub fn add_pat<T: MatcherTrait + Any>(&mut self, src: T) {
        self.matchers.push(src.into());
    }

    pub(crate) fn add_common_pat(&mut self, src: Matcher) {
        self.matchers.push(src);
    }

    pub fn start(&mut self) -> Res<Vec<Token>> {
        if DEBUG {
            dbg!(&self.str_iter);
        }
        let mut res = Vec::with_capacity(self.matchers.len());
        let lookup = LineColLookup::new(&self.str_iter);
        let mut current_str = self.str_iter.clone();
        let mut current_index = 0;
        loop {
            for reg in &self.matchers {
                if let Some(s) = reg.get(&current_str) {
                    current_str = current_str[s.len()..].to_owned();
                    let (line, column) = lookup.get_by_cluster(current_index);
                    current_index += s.len();
                    res.push(Token {
                        val: s,
                        line,
                        column,
                    });
                    break;
                } else {
                    continue;
                }
            }
            if current_str.len() == 0 {
                break;
            }
        }
        Ok(res)
    }
}

#[derive(Debug, Display, Error)]
pub enum TokenizerError {
    AllRegexesMatchNothing,
}

#[inline]
pub fn build_tokenizer<TSrc: ToString>(val: Vec<Matcher>, src: TSrc) -> Tokenizer {
    let mut tokenizer = Tokenizer::new(src);
    for v in val {
        tokenizer.add_common_pat(dbg!(v));
    }
    tokenizer
}

#[inline]
pub fn to_tokens<TSrc: ToString>(val: Vec<Matcher>, src: TSrc) -> Res<Vec<Token>> {
    let mut tokenizer = build_tokenizer(val, src);
    tokenizer.start()
}

#[inline]
pub fn to_tokens_without_ws<TSrc: ToString>(val: Vec<Matcher>, src: TSrc) -> Res<Vec<Token>> {
    let mut tokenizer = build_tokenizer(val, src);
    Ok(filter_white_spaces(tokenizer.start()?))
}

#[inline]
pub fn filter_white_spaces(val: Vec<Token>) -> Vec<Token> {
    if DEBUG {
        dbg!(&val);
    }
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
class A {
}";
        let tokens = filter_white_spaces(
            to_tokens(
                vec![
                    "class".into(),
                    WHITE_SPACE_REGEX.clone().into(),
                    Regex::new(r"\A.+?").unwrap().into(),
                    "{".into(),
                    "}".into(),
                ],
                src,
            )
            .unwrap(),
        );
        dbg!(tokens);
    }
}
