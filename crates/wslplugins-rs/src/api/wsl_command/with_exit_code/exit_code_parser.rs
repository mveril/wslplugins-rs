const EXIT_CODE_OSC_PREFIX: &[u8] = b"\x1b]9;4;";
const BEL: u8 = 0x07;
const MAX_EXIT_CODE_DIGITS: usize = 10;

pub(super) const MAX_EXIT_CODE_SEQUENCE_LEN: usize =
    EXIT_CODE_OSC_PREFIX.len() + MAX_EXIT_CODE_DIGITS + 1;

pub(super) enum PeekedExitCode {
    Complete { exit_code: u32, sequence_len: usize },
    Incomplete,
    NotExitCode,
}

pub(super) fn parse_peeked_exit_code(bytes: &[u8]) -> PeekedExitCode {
    for (index, byte) in bytes.iter().enumerate().take(EXIT_CODE_OSC_PREFIX.len()) {
        if *byte != EXIT_CODE_OSC_PREFIX[index] {
            return PeekedExitCode::NotExitCode;
        }
    }

    if bytes.len() < EXIT_CODE_OSC_PREFIX.len() {
        return PeekedExitCode::Incomplete;
    }

    let mut exit_code = 0_u32;
    let mut digits = 0;

    for (index, byte) in bytes[EXIT_CODE_OSC_PREFIX.len()..].iter().enumerate() {
        if *byte == BEL {
            return if digits == 0 {
                PeekedExitCode::NotExitCode
            } else {
                PeekedExitCode::Complete {
                    exit_code,
                    sequence_len: EXIT_CODE_OSC_PREFIX.len() + index + 1,
                }
            };
        }

        if !byte.is_ascii_digit() || digits == MAX_EXIT_CODE_DIGITS {
            return PeekedExitCode::NotExitCode;
        }

        exit_code = match exit_code
            .checked_mul(10)
            .and_then(|code| code.checked_add(u32::from(byte - b'0')))
        {
            Some(exit_code) => exit_code,
            None => return PeekedExitCode::NotExitCode,
        };
        digits += 1;
    }

    if bytes.len() < MAX_EXIT_CODE_SEQUENCE_LEN {
        PeekedExitCode::Incomplete
    } else {
        PeekedExitCode::NotExitCode
    }
}
