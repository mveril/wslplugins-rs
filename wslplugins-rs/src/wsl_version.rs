use std::{
    fmt::{Debug, Display},
    hash::Hash,
};

#[repr(transparent)]
#[derive(Clone, Eq)]
pub struct WSLVersion(wslpluginapi_sys::WSLVersion);

impl WSLVersion {
    pub fn new(major: u32, minor: u32, revision: u32) -> Self {
        wslpluginapi_sys::WSLVersion {
            Major: major,
            Minor: minor,
            Revision: revision,
        }
        .into()
    }

    pub fn major(&self) -> u32 {
        self.0.Major
    }

    pub fn set_major(&mut self, major: u32) {
        self.0.Major = major
    }

    pub fn minor(&self) -> u32 {
        self.0.Minor
    }

    pub fn set_minor(&mut self, minor: u32) {
        self.0.Minor = minor
    }

    pub fn revision(&self) -> u32 {
        self.0.Revision
    }

    pub fn set_revision(&mut self, revision: u32) {
        self.0.Revision = revision
    }
}

impl From<wslpluginapi_sys::WSLVersion> for WSLVersion {
    fn from(value: wslpluginapi_sys::WSLVersion) -> Self {
        WSLVersion(value)
    }
}

impl From<WSLVersion> for wslpluginapi_sys::WSLVersion {
    fn from(value: WSLVersion) -> Self {
        value.0
    }
}

impl AsRef<WSLVersion> for wslpluginapi_sys::WSLVersion {
    fn as_ref(&self) -> &WSLVersion {
        unsafe { &*(self as *const wslpluginapi_sys::WSLVersion as *const WSLVersion) }
    }
}

impl AsRef<wslpluginapi_sys::WSLVersion> for WSLVersion {
    fn as_ref(&self) -> &wslpluginapi_sys::WSLVersion {
        &self.0
    }
}

impl Hash for WSLVersion {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.major().hash(state);
        self.minor().hash(state);
        self.revision().hash(state);
    }
}

impl Default for WSLVersion {
    fn default() -> Self {
        Self::new(1, 0, 0)
    }
}

impl PartialEq for WSLVersion {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl PartialOrd for WSLVersion {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for WSLVersion {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.0.cmp(&other.0)
    }
}

impl Display for WSLVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{}.{}", self.major(), self.minor(), self.revision())
    }
}

impl Debug for WSLVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct(stringify!(WSLVersion))
            .field("major", &self.major())
            .field("minor", &self.minor())
            .field("revision", &self.revision())
            .finish()
    }
}
