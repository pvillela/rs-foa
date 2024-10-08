use std::{backtrace::Backtrace, error::Error as StdError};

use crate::string;

pub fn extract_boxed<T: StdError + 'static>(
    boxed: Box<dyn StdError + Send + Sync>,
) -> Result<T, Box<dyn StdError + Send + Sync + 'static>> {
    let err_box = boxed.downcast::<T>()?;
    let err: T = *err_box;
    Ok(err)
}

pub fn swap_result<T, E>(f: impl FnOnce() -> std::result::Result<E, T>) -> Result<T, E> {
    match f() {
        Ok(e) => Err(e),
        Err(t) => Ok(t),
    }
}

struct It<'a> {
    curr_source: Option<&'a dyn StdError>,
}

impl<'a> Iterator for It<'a> {
    type Item = &'a dyn StdError;

    fn next(&mut self) -> Option<Self::Item> {
        match self.curr_source {
            None => None,
            Some(err) => {
                let next_src = err.source();
                self.curr_source = next_src;
                Some(err)
            }
        }
    }
}

pub fn source_chain(err: &dyn StdError) -> impl Iterator<Item = &dyn StdError> {
    It {
        curr_source: Some(err),
    }
}

pub fn recursive_msg(err: &(dyn StdError)) -> String {
    let mut chain_iter = source_chain(err);
    let mut buf = String::new();
    let mut closing_buf = String::new();

    let first = chain_iter
        .next()
        .expect("error chain always has a first element");
    buf.push_str(&first.to_string());

    for item in chain_iter {
        buf.push_str(", source_msg=[");
        buf.push_str(&item.to_string());
        closing_buf.push(']');
    }

    buf.push_str(&closing_buf);

    buf
}

// region:      --- StringSpec

#[non_exhaustive]
pub enum StringSpec<'a> {
    Dbg,
    Recursive,
    SourceDbg,
    Backtrace,
    BacktraceDbg,
    Decor(&'a Self, Option<&'a str>, Option<&'a str>),
}

// endregion:   --- StringSpec

pub trait WithBacktrace {
    fn backtrace(&self) -> &Backtrace;
}

pub struct Fmt<'a, T: StdError + WithBacktrace>(pub &'a T);

impl<'a, T: StdError + WithBacktrace> Fmt<'a, T> {
    pub fn dbg_string(&self) -> String {
        format!("{:?}", self.0)
    }

    pub fn recursive_msg(&self) -> String {
        recursive_msg(&self.0)
    }

    pub fn source_dbg_string(&self) -> String {
        format!("{:?}", self.0.source())
    }

    pub fn backtrace_string(&self) -> String {
        format!("{}", self.0.backtrace())
    }

    pub fn backtrace_dbg_string(&self) -> String {
        format!("{:?}", self.0.backtrace())
    }

    pub fn speced_string(&self, str_spec: &StringSpec) -> String {
        match str_spec {
            StringSpec::Dbg => self.dbg_string(),
            StringSpec::Recursive => self.recursive_msg(),
            StringSpec::SourceDbg => self.source_dbg_string(),
            StringSpec::Backtrace => self.backtrace_string(),
            StringSpec::BacktraceDbg => self.backtrace_dbg_string(),
            StringSpec::Decor(&ref spec, pre, post) => {
                string::decorated(&self.speced_string(spec), *pre, *post)
            }
        }
    }

    pub fn multi_speced_string<const N: usize>(&self, str_specs: [StringSpec; N]) -> String {
        let txt = str_specs
            .into_iter()
            .map(|spec| self.speced_string(&spec))
            .collect::<Vec<_>>();
        txt.join(", ")
    }

    pub fn formatted_string(&self, fmt: &str) -> String {
        let props: Vec<(&'static str, fn(&Self) -> String)> = vec![
            ("dbg_string", Self::dbg_string),
            ("recursive_msg", Self::recursive_msg),
            ("source_dbg_string", Self::source_dbg_string),
            ("backtrace_string", Self::backtrace_string),
            ("backtrace_dbg_string", Self::backtrace_dbg_string),
        ];
        string::interpolated_props_lazy(fmt, props.into_iter(), self)
    }

    pub fn speced_string_tuple(&self, str_spec: &StringSpec) -> (&'static str, String) {
        match str_spec {
            StringSpec::Dbg => ("dbg_string", self.dbg_string()),
            StringSpec::Recursive => ("recursive_msg", self.recursive_msg()),
            StringSpec::SourceDbg => ("source_dbg_string", self.source_dbg_string()),
            StringSpec::Backtrace => ("backtrace_string", self.backtrace_string()),
            StringSpec::BacktraceDbg => ("backtrace_dbg_string", self.backtrace_dbg_string()),
            StringSpec::Decor(&ref spec, _, _) => self.speced_string_tuple(spec),
        }
    }
}
