use serde::Serialize;
use std::fmt::{Debug, Display};
use std::ops::Deref;

//===========================
// region:      --- StaticStr

pub(super) enum StaticStr {
    Ref(&'static str),
    Owned(String),
}

impl From<&'static str> for StaticStr {
    fn from(value: &'static str) -> Self {
        Self::Ref(value)
    }
}

impl From<String> for StaticStr {
    fn from(value: String) -> Self {
        Self::Owned(value)
    }
}

impl AsRef<str> for StaticStr {
    fn as_ref(&self) -> &str {
        match self {
            Self::Ref(txt) => txt,
            Self::Owned(txt) => txt,
        }
    }
}

impl Deref for StaticStr {
    type Target = str;

    fn deref(&self) -> &str {
        match self {
            Self::Ref(txt) => txt,
            Self::Owned(txt) => txt,
        }
    }
}

impl Clone for StaticStr {
    fn clone(&self) -> Self {
        match self {
            Self::Ref(txt) => Self::Ref(txt),
            Self::Owned(txt) => Self::Owned(txt.clone()),
        }
    }
}

impl Debug for StaticStr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Debug::fmt(self.as_ref(), f)
    }
}

impl Display for StaticStr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Display::fmt(self.as_ref(), f)
    }
}

impl PartialEq for StaticStr {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Ref(myself), Self::Ref(other)) if myself == other => true,
            (Self::Owned(myself), Self::Owned(other)) if myself == other => true,
            _ => false,
        }
    }
}

impl Eq for StaticStr {}

impl Serialize for StaticStr {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            Self::Ref(txt) => txt.serialize(serializer),
            Self::Owned(txt) => txt.serialize(serializer),
        }
    }
}

// endregion:   --- StaticStr

#[cfg(test)]
mod test {
    use crate::error::static_str::StaticStr;

    #[test]
    fn test_staticstr_str() {
        let txt = "abc";

        let exp_to_string = txt.to_owned();
        let exp_display = format!("{txt}");
        let exp_debug = format!("{txt:?}");
        let exp_ser = r#""abc""#;

        let statstr = StaticStr::from(txt);
        let to_string = statstr.to_string();
        let display = format!("{statstr}");
        let debug = format!("{statstr:?}");
        let ser = serde_json::to_string(&statstr).unwrap();

        assert_eq!(to_string, exp_to_string);
        assert_eq!(display, exp_display);
        assert_eq!(debug, exp_debug);
        assert_eq!(ser, exp_ser);
    }

    #[test]
    fn test_staticstr_string() {
        let txt = "def".to_owned();

        let exp_to_string = txt.clone();
        let exp_display = format!("{txt}");
        let exp_debug = format!("{txt:?}");
        let exp_ser = r#""def""#;

        let statstr = StaticStr::from(txt);
        let to_string = statstr.to_string();
        let display = format!("{statstr}");
        let debug = format!("{statstr:?}");
        let ser = serde_json::to_string(&statstr).unwrap();

        assert_eq!(to_string, exp_to_string);
        assert_eq!(display, exp_display);
        assert_eq!(debug, exp_debug);
        assert_eq!(ser, exp_ser);
    }
}
