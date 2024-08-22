use std::{any::Any, fmt::{Debug, Display, Formatter}, io::{stdout, BufWriter}};

use regex::Regex;

pub trait MatcherTrait {
    fn get(&self, src: &String) -> Option<String>;
    fn get_at(&self, src: &String, index: usize) -> Option<String>;
    fn _to_string(&self) -> String;
}

impl MatcherTrait for String {
    fn get(&self, src: &String) -> Option<String> {
        if src.starts_with(self) {
            Some(self.clone())
        } else {
            None
        }
    }
    fn get_at(&self, src: &String, index: usize) -> Option<String> {
        if let Some(x) = src.match_indices(self).find(|x| x.0 == index) {
            Some(x.1.to_string())
        } else {
            None
        }
    }
    fn _to_string(&self) -> String {
        self.clone()
    }
}

impl MatcherTrait for Vec<String> {
    fn get(&self, src: &String) -> Option<String> {
        for v in self {
            if let Some(x) = v.get(src) {
                return Some(x);
            }
        }
        None
    }
    fn get_at(&self, src: &String, index: usize) -> Option<String> {
        for v in self {
            if let Some(x) = v.get_at(src, index) {
                return Some(x);
            }
        }
        None
    }
    fn _to_string(&self) -> String {
        serde_json::to_string(self).unwrap()
    }
}

impl MatcherTrait for str {
    fn get(&self, src: &String) -> Option<String> {
        self.to_owned().get(src)
    }
    fn get_at(&self, src: &String, index: usize) -> Option<String> {
        self.to_owned().get_at(src, index)
    }
    fn _to_string(&self) -> String {
        self.to_owned()
    }
}

impl MatcherTrait for &str {
    fn get(&self, src: &String) -> Option<String> {
        (*self).to_owned().get(src)
    }
    fn get_at(&self, src: &String, index: usize) -> Option<String> {
        (*self).to_owned().get_at(src, index)
    }
    fn _to_string(&self) -> String {
        (*self).to_owned()
    }
}

impl<T> MatcherTrait for T
where
    T: Fn(&String) -> Option<String>,
{
    fn get(&self, src: &String) -> Option<String> {
        self(src)
    }
    fn get_at(&self, src: &String, index: usize) -> Option<String> {
        self(&src[index..].to_owned())
    }
    fn _to_string(&self) -> String {
        "<Fn>".to_owned()
    }
}

impl MatcherTrait for Regex {
    fn get(&self, src: &String) -> Option<String> {
        if let Some(s) = self.captures(src.as_str()) {
            Some(s.get(0).unwrap().as_str().to_string())
        } else {
            None
        }
    }
    fn get_at(&self, src: &String, index: usize) -> Option<String> {
        if let Some(x) = self.captures_at(src.as_str(), index) {
            Some(x.get(0).unwrap().as_str().to_string())
        } else {
            None
        }
    }
    fn _to_string(&self) -> String {
        self.to_string()
    }
}

pub struct Matcher {
    inner: Box<dyn MatcherTrait>,
}

impl Display for Matcher {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.inner._to_string())
    }
}

impl Debug for Matcher {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        <Self as Display>::fmt(&self, f)
    }
}

impl Matcher {
    pub fn get<T: ToString>(&self, src: T) -> Option<String> {
        self.inner.get(&src.to_string())
    }
    pub fn get_at<T: ToString>(&self, src: T, index: usize) -> Option<String> {
        self.inner.get_at(&src.to_string(), index)
    }
}

impl<T: MatcherTrait + Any> From<T> for Matcher {
    fn from(value: T) -> Self {
        Self {
            inner: Box::new(value),
        }
    }
}
