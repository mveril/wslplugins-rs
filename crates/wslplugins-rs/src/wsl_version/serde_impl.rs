use std::fmt;

use serde::{
    de::{Error, Visitor},
    Deserialize, Deserializer, Serialize,
};

use crate::WSLVersion;

impl Serialize for WSLVersion {
    #[inline]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.collect_str(self)
    }
}

impl<'de> Deserialize<'de> for WSLVersion {
    #[inline]
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct WSLVersionVisitor;

        impl Visitor<'_> for WSLVersionVisitor {
            type Value = WSLVersion;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("WSL version")
            }

            fn visit_str<E>(self, string: &str) -> Result<Self::Value, E>
            where
                E: Error,
            {
                string.parse().map_err(Error::custom)
            }
        }

        deserializer.deserialize_str(WSLVersionVisitor)
    }
}

#[cfg(test)]
mod tests {
    use crate::WSLVersion;
    use serde_test::{assert_tokens, Configure, Token};

    #[test]
    fn serde_version_as_semver_string() {
        let version = WSLVersion::new(2, 4, 4);
        assert_tokens(&version.readable(), &[Token::Str("2.4.4")]);
    }
}
