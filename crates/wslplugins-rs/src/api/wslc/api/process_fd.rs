#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[repr(i32)]
pub enum WSLCProcessFd {
    /// Standard input.
    Stdin = wslpluginapi_sys::WSLCProcessFd_WSLCProcessFdStdin,
    /// Standard output.
    Stdout = wslpluginapi_sys::WSLCProcessFd_WSLCProcessFdStdout,
    /// Standard error.
    Stderr = wslpluginapi_sys::WSLCProcessFd_WSLCProcessFdStderr,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct WSLCProcessStreams(u8);

impl WSLCProcessStreams {
    pub(crate) const STDIN: Self = Self(1 << 0);
    pub(crate) const STDOUT: Self = Self(1 << 1);
    pub(crate) const STDERR: Self = Self(1 << 2);
    pub(crate) const ALL: Self = Self(Self::STDIN.0 | Self::STDOUT.0 | Self::STDERR.0);

    #[inline]
    pub(crate) fn insert(&mut self, streams: Self) {
        self.0 |= streams.0;
    }

    #[inline]
    pub(crate) const fn contains(self, streams: Self) -> bool {
        self.0 & streams.0 == streams.0
    }

    #[inline]
    pub(crate) const fn union(self, streams: Self) -> Self {
        Self(self.0 | streams.0)
    }
}

#[cfg(test)]
mod tests {
    use super::WSLCProcessStreams;
    use std::mem::size_of;

    fn stream_combinations() -> [WSLCProcessStreams; 8] {
        [
            WSLCProcessStreams::default(),
            WSLCProcessStreams::STDIN,
            WSLCProcessStreams::STDOUT,
            WSLCProcessStreams::STDIN.union(WSLCProcessStreams::STDOUT),
            WSLCProcessStreams::STDERR,
            WSLCProcessStreams::STDIN.union(WSLCProcessStreams::STDERR),
            WSLCProcessStreams::STDOUT.union(WSLCProcessStreams::STDERR),
            WSLCProcessStreams::ALL,
        ]
    }

    #[test]
    fn process_streams_use_one_byte_and_expected_bit_values() {
        assert_eq!(size_of::<WSLCProcessStreams>(), 1);
        assert_eq!(WSLCProcessStreams::default(), WSLCProcessStreams(0b000));
        assert_eq!(WSLCProcessStreams::STDIN, WSLCProcessStreams(0b001));
        assert_eq!(WSLCProcessStreams::STDOUT, WSLCProcessStreams(0b010));
        assert_eq!(WSLCProcessStreams::STDERR, WSLCProcessStreams(0b100));
        assert_eq!(WSLCProcessStreams::ALL, WSLCProcessStreams(0b111));
    }

    #[test]
    fn every_stream_combination_has_the_expected_members() {
        for (bits, streams) in stream_combinations().into_iter().enumerate() {
            for (flag, expected) in [
                (WSLCProcessStreams::STDIN, bits & 0b001 != 0),
                (WSLCProcessStreams::STDOUT, bits & 0b010 != 0),
                (WSLCProcessStreams::STDERR, bits & 0b100 != 0),
            ] {
                assert_eq!(
                    streams.contains(flag),
                    expected,
                    "streams={bits:03b}, flag={:03b}",
                    flag.0,
                );
            }
        }
    }

    #[test]
    fn operations_are_correct_for_every_pair_of_stream_combinations() {
        for (left_bits, left) in stream_combinations().into_iter().enumerate() {
            assert!(left.contains(WSLCProcessStreams::default()));
            assert_eq!(left.union(WSLCProcessStreams::default()), left);
            assert_eq!(left.union(left), left);

            for (right_bits, right) in stream_combinations().into_iter().enumerate() {
                let expected = stream_combinations()[left_bits | right_bits];
                let mut inserted = left;
                inserted.insert(right);

                assert_eq!(
                    inserted, expected,
                    "insert: left={left_bits:03b}, right={right_bits:03b}"
                );
                assert_eq!(
                    left.union(right),
                    expected,
                    "union: left={left_bits:03b}, right={right_bits:03b}",
                );
                assert_eq!(
                    left.union(right),
                    right.union(left),
                    "commutativity: left={left_bits:03b}, right={right_bits:03b}",
                );
                assert_eq!(
                    left.contains(right),
                    left_bits & right_bits == right_bits,
                    "contains: left={left_bits:03b}, right={right_bits:03b}",
                );
            }
        }
    }

    #[test]
    fn union_is_associative_for_every_triplet_of_stream_combinations() {
        for (left_bits, left) in stream_combinations().into_iter().enumerate() {
            for (middle_bits, middle) in stream_combinations().into_iter().enumerate() {
                for (right_bits, right) in stream_combinations().into_iter().enumerate() {
                    assert_eq!(
                        left.union(middle).union(right),
                        left.union(middle.union(right)),
                        "left={left_bits:03b}, middle={middle_bits:03b}, right={right_bits:03b}",
                    );
                }
            }
        }
    }
}
