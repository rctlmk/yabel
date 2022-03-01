use std::borrow::Cow;
use std::str;

use crate::items::*;
use crate::DecodeError;
use crate::ErrorKind::*;

#[non_exhaustive]
#[derive(Debug)]
/// Decoder settings.
pub enum Settings {
    /// Allow only sorted dictionaries.
    SortedDictionaries,
    /// Allow sorted and unsorted dictionaries.
    UnsortedDictionaries,
}

/// Bencode decoder.
pub struct Decoder<'a> {
    bytes: &'a [u8],
    cursor: usize,
    allow_unsorted_dictionaries: bool,
}

impl<'a> Decoder<'a> {
    /// Constructs a new `Decoder` with specified byte buffer.
    pub fn new(bytes: &'a [u8]) -> Self {
        Self {
            bytes,
            cursor: 0,
            allow_unsorted_dictionaries: false,
        }
    }

    /// Applies a setting for the current decoder.
    ///
    /// See [`Settings`] enum for a full list.
    pub fn setting(self, setting: Settings) -> Self {
        let mut s = self;

        match setting {
            Settings::SortedDictionaries => s.allow_unsorted_dictionaries = false,
            Settings::UnsortedDictionaries => s.allow_unsorted_dictionaries = true,
        }

        s
    }

    /// Decodes items.
    pub fn decode(&mut self) -> Result<Vec<Item<'a>>, DecodeError> {
        let mut items = vec![];

        while let Some(byte) = self.bytes.get(self.cursor) {
            items.push(self.decode_item(byte)?);
        }

        Ok(items)
    }

    /// Decodes a single `Item`.
    ///
    /// # Error
    ///
    /// See [`DecodeError`] and [`ErrorKind`](crate::ErrorKind) for more details.
    fn decode_item(&mut self, byte: &u8) -> Result<Item<'a>, DecodeError> {
        match byte {
            b'0'..=b'9' => Ok(Item::String(self.decode_string()?)),
            b'i' => Ok(Item::Integer(self.decode_integer()?)),
            b'l' => Ok(Item::List(self.decode_list()?)),
            b'd' => Ok(Item::Dictionary(self.decode_dictionary()?)),
            b => {
                Err(DecodeError {
                    kind: UnexpectedByte(*b),
                })
            },
        }
    }

    /// Reads bytes from the buffer until `stop_byte` is reached and returns the read bytes.
    ///
    /// # Errors
    ///
    /// Returns [`UnexpectedEndOfBuffer`] if `stop_byte` was not reached.
    fn read_bytes(&mut self, stop_byte: u8) -> Result<&[u8], DecodeError> {
        self.bytes
            .iter()
            .skip(self.cursor)
            .position(|b| b == &stop_byte)
            .ok_or(DecodeError {
                kind: UnexpectedEndOfBuffer,
            })
            .map(|pos| {
                let pos = pos + self.cursor;

                let bytes = &self.bytes[self.cursor..pos];

                self.cursor = pos + 1;

                bytes
            })
    }

    /// Decodes a string.
    fn decode_string(&mut self) -> Result<BString<'a>, DecodeError> {
        self.read_bytes(b':')
            .and_then(parse_i64)
            .map(|length| length as usize)
            .and_then(|length| {
                let s = self
                    .bytes
                    .get(self.cursor..self.cursor + length)
                    .ok_or(DecodeError {
                        kind: UnexpectedEndOfBuffer,
                    })
                    .map(|s| BString(Cow::from(s)));

                self.cursor += length;

                s
            })
    }

    /// Decodes an integer.
    fn decode_integer(&mut self) -> Result<BInteger, DecodeError> {
        self.cursor += 1;

        self.read_bytes(b'e').and_then(parse_i64).map(BInteger)
    }

    /// Decodes a list.
    fn decode_list(&mut self) -> Result<BList<'a>, DecodeError> {
        self.cursor += 1;

        let mut items = vec![];

        let mut decode_is_done = false;

        while let Some(byte) = self.bytes.get(self.cursor) {
            if *byte == b'e' {
                decode_is_done = true;
                break;
            };
            items.push(self.decode_item(byte)?);
        }

        self.cursor += 1;

        if decode_is_done {
            Ok(BList(items))
        } else {
            Err(DecodeError {
                kind: UnexpectedEndOfBuffer,
            })
        }
    }

    /// Decodes a dictionary.
    fn decode_dictionary(&mut self) -> Result<BDictionary<'a>, DecodeError> {
        self.cursor += 1;

        let mut items = vec![];

        let mut decode_is_done = false;

        while let Some(byte) = self.bytes.get(self.cursor).cloned() {
            if byte == b'e' {
                decode_is_done = true;
                break;
            };

            let key = self.decode_item(&byte)?.string().ok_or(DecodeError {
                kind: InvalidDictionaryKey,
            })?;

            if !self.allow_unsorted_dictionaries && items.last().map_or(false, |(k, _)| k > &key) {
                return Err(DecodeError {
                    kind: UnsortedDictionary,
                });
            }

            let byte = self.bytes.get(self.cursor).ok_or(DecodeError {
                kind: UnexpectedEndOfBuffer,
            })?;

            items.push((key, self.decode_item(byte)?));
        }

        self.cursor += 1;

        if decode_is_done {
            Ok(BDictionary(items.into_iter().collect()))
        } else {
            Err(DecodeError {
                kind: UnexpectedEndOfBuffer,
            })
        }
    }
}

/// Parses an integer from byte slice.
fn parse_i64(bytes: &[u8]) -> Result<i64, DecodeError> {
    match bytes[..] {
        [b'-', b'0', _, ..] | [b'0', _, ..] => Err(DecodeError { kind: LeadingZeros }),
        [b'-', b'0', ..] => Err(DecodeError { kind: NegativeZero }),
        _ => {
            str::from_utf8(bytes)
                .map_err(|_e| DecodeError { kind: InvalidData })
                .and_then(|s| s.parse().map_err(|_e| DecodeError { kind: InvalidData }))
        },
    }
}

#[cfg(test)]
mod tests {
    use std::{str, vec};

    use crate::items::*;
    use crate::ErrorKind::*;
    use crate::{DecodeError, Decoder, Settings};

    fn process_string(expected: &str) {
        let input = format!("{}:{}", expected.len(), &expected);

        let v = Decoder::new(input.as_bytes()).decode().unwrap();
        let actual = v.into_iter().next().unwrap().string().unwrap();

        assert_eq!(expected, str::from_utf8(&actual.0).unwrap());
    }

    fn process_list(input: &str, expected: Vec<Item>) {
        let v = Decoder::new(input.as_bytes()).decode().unwrap();
        let actual = v.into_iter().next().unwrap().list().unwrap();

        assert_eq!(
            expected.len(),
            actual.0.len(),
            "expected length = {}, actual length = {}",
            expected.len(),
            actual.0.len()
        );

        expected
            .into_iter()
            .zip(actual.0.into_iter())
            .for_each(|(expected, actual)| assert_eq!(expected, actual))
    }

    fn process_dictionary(input: &str, expected_pairs: Vec<(BString, Item)>) {
        let v = Decoder::new(input.as_bytes()).decode().unwrap();
        let actual = v.into_iter().next().unwrap().dictionary().unwrap();

        let actual_pairs: Vec<_> = actual.0.into_iter().collect();
        assert_eq!(
            expected_pairs.len(),
            actual_pairs.len(),
            "expected length = {}, actual length = {}",
            expected_pairs.len(),
            actual_pairs.len()
        );

        expected_pairs
            .into_iter()
            .zip(actual_pairs.into_iter())
            .for_each(|((k1, v1), (k2, v2))| {
                assert_eq!(k1, k2);
                assert_eq!(v1, v2);
            })
    }

    #[test]
    fn short_string() {
        process_string("test")
    }

    #[test]
    fn long_string() {
        process_string("sixteencharslong")
    }

    #[test]
    fn empty_string() {
        process_string("")
    }

    #[test]
    fn string_with_incorrect_length() {
        let input = b"7:foo";

        assert_eq!(
            Decoder::new(&input[..]).decode(),
            Err(DecodeError {
                kind: UnexpectedEndOfBuffer
            })
        );
    }

    #[test]
    fn two_strings_in_a_row() {
        let input = b"3:foo4:barr";

        assert!(Decoder::new(&input[..]).decode().is_ok());
    }

    #[test]
    fn negative_zero() {
        let input = b"i-0e";

        assert_eq!(Decoder::new(&input[..]).decode(), Err(DecodeError { kind: NegativeZero }));
    }

    #[test]
    fn integer() {
        let expected = 1234567890_i64;

        let input = format!("i{}e", expected);

        let v = Decoder::new(input.as_bytes()).decode().unwrap();
        let actual = v.into_iter().next().unwrap().integer().unwrap();

        assert_eq!(expected, actual.0);
    }

    #[test]
    fn minus() {
        let input = b"i-e";

        assert_eq!(Decoder::new(&input[..]).decode(), Err(DecodeError { kind: InvalidData }));
    }

    #[test]
    fn empty_integer() {
        let input = b"ie";

        assert_eq!(Decoder::new(&input[..]).decode(), Err(DecodeError { kind: InvalidData }));
    }

    #[test]
    fn integer_with_leading_zeros() {
        let input = b"i001e";

        assert_eq!(Decoder::new(&input[..]).decode(), Err(DecodeError { kind: LeadingZeros }));
    }

    #[test]
    fn malformed_integer() {
        let input = b"i-4AF54e";

        assert_eq!(Decoder::new(&input[..]).decode(), Err(DecodeError { kind: InvalidData }));
    }

    #[test]
    fn empty_list() {
        process_list("le", vec![])
    }

    #[test]
    fn nested_list() {
        process_list("llleee", vec![vec![vec![].into()].into()])
    }

    #[test]
    fn simple_list() {
        process_list("li17e3:foo3:bare", vec![17.into(), "foo".into(), "bar".into()])
    }

    #[test]
    fn malformed_lists() {
        let input = vec!["l4e", "l0:", "l3:gge", "li00002ee"];

        input.iter().for_each(|s| assert!(Decoder::new(s.as_bytes()).decode().is_err()))
    }

    #[test]
    fn empty_dictionary() {
        process_dictionary("de", vec![])
    }

    #[test]
    fn simple_dictionary() {
        let items = vec![("bar".into(), "spam".into()), ("foo".into(), 42.into())];

        process_dictionary("d3:bar4:spam3:fooi42ee", items)
    }

    #[test]
    fn unsorted_dictionary() {
        assert!(Decoder::new("d2:ccle2:bblee".as_bytes())
            .setting(Settings::UnsortedDictionaries)
            .decode()
            .is_ok());
    }

    #[test]
    fn unsorted_dictionary_without_settings() {
        let res = Decoder::new("d2:ccle2:bblee".as_bytes()).decode();
        assert_eq!(
            res,
            Err(DecodeError {
                kind: UnsortedDictionary
            })
        );
    }
}
