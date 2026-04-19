//! Serde support for [`crate::UserDistributionID`].
//!
//! Human-readable serializers use the canonical GUID string representation.
//! Compact serializers use the native 16-byte Windows GUID memory layout rather
//! than RFC 4122 byte order so the encoded bytes match Windows API expectations.

use std::{ptr, slice};
use windows_core::GUID;

use crate::UserDistributionID;

#[inline]
const fn guid_to_windows_bytes(guid: &windows_core::GUID) -> &[u8] {
    // SAFETY: The layout of windows_core::GUID is guaranteed to be 16 bytes and match the Windows GUID layout
    unsafe { slice::from_raw_parts(std::ptr::from_ref::<GUID>(guid).cast::<u8>(), 16) }
}

#[inline]
const fn guid_from_windows_bytes(bytes: &[u8; 16]) -> GUID {
    // SAFETY: We know guid is 16 bytes and the caller guarantees the bytes are a valid Windows [GUID] representation
    unsafe { ptr::read_unaligned(bytes.as_ptr().cast::<GUID>()) }
}

impl serde::Serialize for UserDistributionID {
    #[inline]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        if serializer.is_human_readable() {
            serializer.collect_str(self)
        } else {
            serializer.serialize_bytes(guid_to_windows_bytes(&self.0))
        }
    }
}

impl<'de> serde::Deserialize<'de> for UserDistributionID {
    #[inline]
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct UserDistributionIDVisitor;

        impl UserDistributionIDVisitor {
            fn from_str<E>(value: &str) -> Result<UserDistributionID, E>
            where
                E: serde::de::Error,
            {
                value.parse::<UserDistributionID>().map_err(E::custom)
            }

            fn from_bytes<E>(bytes: &[u8]) -> Result<UserDistributionID, E>
            where
                E: serde::de::Error,
            {
                if bytes.len() != 16 {
                    return Err(E::invalid_length(bytes.len(), &"a 16-byte GUID"));
                }

                let bytes: [u8; 16] = bytes
                    .try_into()
                    .map_err(|_| E::custom("invalid GUID format in byte array"))?;
                Ok(UserDistributionID(guid_from_windows_bytes(&bytes)))
            }

            fn from_seq<'de, A>(mut seq: A) -> Result<UserDistributionID, A::Error>
            where
                A: serde::de::SeqAccess<'de>,
            {
                let mut bytes = [0u8; 16];
                for (index, slot) in bytes.iter_mut().enumerate() {
                    *slot = seq.next_element()?.ok_or_else(|| {
                        serde::de::Error::invalid_length(index, &"a 16-byte GUID")
                    })?;
                }

                if seq.next_element::<u8>()?.is_some() {
                    return Err(serde::de::Error::invalid_length(17, &"a 16-byte GUID"));
                }

                Ok(UserDistributionID(guid_from_windows_bytes(&bytes)))
            }
        }

        impl<'de> serde::de::Visitor<'de> for UserDistributionIDVisitor {
            type Value = UserDistributionID;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("a GUID string or 16-byte GUID")
            }

            fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Self::from_str(value)
            }

            fn visit_bytes<E>(self, value: &[u8]) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Self::from_bytes(value)
            }

            fn visit_seq<A>(self, seq: A) -> Result<Self::Value, A::Error>
            where
                A: serde::de::SeqAccess<'de>,
            {
                Self::from_seq(seq)
            }
        }

        if deserializer.is_human_readable() {
            deserializer.deserialize_str(UserDistributionIDVisitor)
        } else {
            deserializer.deserialize_bytes(UserDistributionIDVisitor)
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::UserDistributionID;
    use serde_test::{assert_tokens, Configure, Token};
    use std::str::FromStr;
    use windows_core::GUID;

    #[test]
    fn serde_roundtrip_uses_guid_string() {
        let value = UserDistributionID(GUID::from_u128(0x12345678_9abc_def0_1357_2468ace0bdf1));
        assert_tokens(
            &value.readable(),
            &[Token::Str("12345678-9ABC-DEF0-1357-2468ACE0BDF1")],
        );
    }

    #[test]
    fn serde_compact_uses_windows_guid_bytes() {
        let value = UserDistributionID(GUID::from_u128(0x12345678_9abc_def0_1357_2468ace0bdf1));

        assert_tokens(
            &value.compact(),
            &[Token::BorrowedBytes(&[
                0x78, 0x56, 0x34, 0x12, 0xbc, 0x9a, 0xf0, 0xde, 0x13, 0x57, 0x24, 0x68, 0xac, 0xe0,
                0xbd, 0xf1,
            ])],
        );
    }

    #[test]
    fn binary_layout_matches_windows_guid_layout() {
        let guid = GUID::from_u128(0x12345678_9abc_def0_1357_2468ace0bdf1);
        assert_eq!(
            super::guid_to_windows_bytes(&guid),
            [
                0x78, 0x56, 0x34, 0x12, 0xbc, 0x9a, 0xf0, 0xde, 0x13, 0x57, 0x24, 0x68, 0xac, 0xe0,
                0xbd, 0xf1,
            ]
        );
    }

    #[test]
    fn binary_deserialization_uses_windows_guid_layout() {
        let bytes = [
            0x78, 0x56, 0x34, 0x12, 0xbc, 0x9a, 0xf0, 0xde, 0x13, 0x57, 0x24, 0x68, 0xac, 0xe0,
            0xbd, 0xf1,
        ];
        let guid = super::guid_from_windows_bytes(&bytes);
        assert_eq!(
            guid,
            GUID::from_u128(0x12345678_9abc_def0_1357_2468ace0bdf1)
        );
    }

    #[test]
    fn binary_layout_matches_expected_bytes_for_known_guid() {
        #[allow(clippy::unwrap_used)]
        let id = UserDistributionID::from_str("80E4258D-0E16-4301-B8BE-E7833D02A7AA").unwrap();

        assert_eq!(
            super::guid_to_windows_bytes(&id.0),
            [141, 37, 228, 128, 22, 14, 1, 67, 184, 190, 231, 131, 61, 2, 167, 170,]
        );
    }
}
