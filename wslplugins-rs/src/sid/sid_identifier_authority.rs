#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct SidIdentifierAuthority {
    pub value: [u8; 6],
}
impl Default for SidIdentifierAuthority {
    fn default() -> Self {
        unsafe { core::mem::zeroed() }
    }
}

impl From<[u8; 6]> for SidIdentifierAuthority {
    fn from(value: [u8; 6]) -> Self {
        Self { value }
    }
}

impl From<SidIdentifierAuthority> for [u8; 6] {
    fn from(value: SidIdentifierAuthority) -> Self {
        value.value
    }
}

#[cfg(test)]
pub(crate) mod test {
    use super::*;
    use proptest::prelude::*;
    prop_compose! {
        pub fn arb_identifier_authority()
            (val in 1u8..=5)
            -> SidIdentifierAuthority {
            // les 5 octets supérieurs sont toujours 0, seul le dernier peut varier
            let mut bytes = [0u8; 6];
            bytes[5] = val;
            SidIdentifierAuthority::from(bytes)
        }
    }
}
