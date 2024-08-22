use std::cell::LazyCell;

use regex::Regex;

pub(crate) const WHITE_SPACE: LazyCell<Vec<&'static str>> = LazyCell::new(|| {
    vec!["\n", "\r", "\t", " "]
});

pub(crate) const WHITE_SPACE_REGEX: LazyCell<Regex> = LazyCell::new(|| {
    Regex::new(r"\A\s").unwrap()
});

#[cfg(debug_assertions)]
pub(crate) const DEBUG: bool = true;

#[cfg(not(debug_assertions))]
pub(crate) const DEBUG: bool = false;
