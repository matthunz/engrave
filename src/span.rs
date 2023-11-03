use std::rc::Rc;

/// Span of text with an optional node kind.
#[derive(Clone, Debug, Eq)]
pub struct Span {
    pub kind: Option<Rc<str>>,
    pub text: Rc<str>,
}

impl Span {
    pub fn new(kind: Option<Rc<str>>, text: impl Into<Rc<str>>) -> Self {
        Self {
            kind,
            text: text.into(),
        }
    }

    pub fn from_text(text: impl Into<Rc<str>>) -> Self {
        Self::new(None, text)
    }

    pub fn from_kind(kind: impl Into<Rc<str>>, text: impl Into<Rc<str>>) -> Self {
        Self::new(Some(kind.into()), text)
    }
}

impl PartialEq for Span {
    fn eq(&self, other: &Self) -> bool {
        self.kind == other.kind && Rc::ptr_eq(&self.text, &other.text)
    }
}
