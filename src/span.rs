use std::rc::Rc;

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

    pub fn text(text: impl Into<Rc<str>>) -> Self {
        Self::new(None, text)
    }
}

impl PartialEq for Span {
    fn eq(&self, other: &Self) -> bool {
        self.kind == other.kind && Rc::ptr_eq(&self.text, &other.text)
    }
}
