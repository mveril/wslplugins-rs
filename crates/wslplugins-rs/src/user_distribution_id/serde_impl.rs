use crate::UserDistributionID;

#[inline]
fn guid_to_windows_bytes(guid: windows_core::GUID) -> [u8; 16] {
    let mut bytes = [0_u8; 16];
    bytes[..4].copy_from_slice(&guid.data1.to_le_bytes());
    bytes[4..6].copy_from_slice(&guid.data2.to_le_bytes());
    bytes[6..8].copy_from_slice(&guid.data3.to_le_bytes());
    bytes[8..].copy_from_slice(&guid.data4);
    bytes
}

#[inline]
fn guid_from_windows_bytes(bytes: &[u8; 16]) -> windows_core::GUID {
    let data1 = [bytes[0], bytes[1], bytes[2], bytes[3]];
    let data2 = [bytes[4], bytes[5]];
    let data3 = [bytes[6], bytes[7]];
    let data4 = [
        bytes[8], bytes[9], bytes[10], bytes[11], bytes[12], bytes[13], bytes[14], bytes[15],
    ];

    windows_core::GUID {
        data1: u32::from_le_bytes(data1),
        data2: u16::from_le_bytes(data2),
        data3: u16::from_le_bytes(data3),
        data4,
    }
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
            serializer.serialize_bytes(&guid_to_windows_bytes(self.0))
        }
    }
}

impl<'de> serde::Deserialize<'de> for UserDistributionID {
    #[inline]
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        if deserializer.is_human_readable() {
            let value =
                <std::borrow::Cow<'de, str> as serde::Deserialize>::deserialize(deserializer)?;
            value.parse::<Self>().map_err(serde::de::Error::custom)
        } else {
            let bytes = <&[u8] as serde::Deserialize>::deserialize(deserializer)?;
            if bytes.len() != 16 {
                return Err(serde::de::Error::invalid_length(
                    bytes.len(),
                    &"a 16-byte GUID",
                ));
            }

            let bytes: [u8; 16] = bytes
                .try_into()
                .map_err(|_| serde::de::Error::custom("invalid GUID format in byte array"))?;
            Ok(Self(guid_from_windows_bytes(&bytes)))
        }
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use crate::UserDistributionID;

    #[test]
    fn serde_roundtrip_uses_guid_string() {
        let value = UserDistributionID(windows_core::GUID::from_u128(
            0x12345678_9abc_def0_1357_2468ace0bdf1,
        ));
        let json = serde_json::to_string(&value).unwrap();
        assert_eq!(json, "\"12345678-9ABC-DEF0-1357-2468ACE0BDF1\"");
        let decoded: UserDistributionID = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded, value);
    }

    #[test]
    fn binary_layout_matches_windows_guid_layout() {
        let guid = windows_core::GUID::from_u128(0x12345678_9abc_def0_1357_2468ace0bdf1);
        assert_eq!(
            super::guid_to_windows_bytes(guid),
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
            windows_core::GUID::from_u128(0x12345678_9abc_def0_1357_2468ace0bdf1)
        );
    }

    #[test]
    fn binary_layout_matches_expected_bytes_for_known_guid() {
        let id = UserDistributionID::from_str("80E4258D-0E16-4301-B8BE-E7833D02A7AA").unwrap();

        assert_eq!(
            super::guid_to_windows_bytes(id.0),
            [141, 37, 228, 128, 22, 14, 1, 67, 184, 190, 231, 131, 61, 2, 167, 170,]
        );
    }
}
