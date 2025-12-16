use std::borrow::Cow;
use typed_path::{Utf8UnixPath, Utf8UnixPathBuf};
pub trait IntoCowUtf8UnixPath<'a> {
    fn into_cow_utf8_unix_path(self) -> Cow<'a, Utf8UnixPath>;
}

impl<'a> IntoCowUtf8UnixPath<'a> for Cow<'a, str> {
    fn into_cow_utf8_unix_path(self) -> Cow<'a, Utf8UnixPath> {
        match self {
            Cow::Borrowed(s) => Cow::Borrowed(&s.as_ref()),
            Cow::Owned(s) => Cow::Owned(s.into()),
        }
    }
}

impl<'a> IntoCowUtf8UnixPath<'a> for &'a str {
    fn into_cow_utf8_unix_path(self) -> Cow<'a, Utf8UnixPath> {
        Cow::Borrowed(&self.as_ref())
    }
}

impl<'a> IntoCowUtf8UnixPath<'a> for String {
    fn into_cow_utf8_unix_path(self) -> Cow<'a, Utf8UnixPath> {
        Cow::Owned(self.into())
    }
}

impl<'a> IntoCowUtf8UnixPath<'a> for &'a Utf8UnixPath {
    fn into_cow_utf8_unix_path(self) -> Cow<'a, Utf8UnixPath> {
        self.into()
    }
}

impl<'a> IntoCowUtf8UnixPath<'a> for Utf8UnixPathBuf {
    fn into_cow_utf8_unix_path(self) -> Cow<'a, Utf8UnixPath> {
        self.into()
    }
}
