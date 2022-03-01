use std::borrow::Cow;
use std::collections::BTreeMap;
use std::fmt::{Debug, Display};
use std::{fmt, str};

use crate::encode::Bencode;

#[derive(Debug, Eq, PartialEq, Clone)]
/// The item type.
pub enum Item<'a> {
    /// Byte string.
    String(BString<'a>),
    /// Integer.
    Integer(BInteger),
    /// List.
    List(BList<'a>),
    /// Dictionary.
    Dictionary(BDictionary<'a>),
}

#[derive(Default, Ord, PartialOrd, PartialEq, Eq, Clone)]
/// The byte string type.
pub struct BString<'a>(pub Cow<'a, [u8]>);

#[derive(Default, Debug, Eq, PartialEq, Clone)]
/// The integer type.
pub struct BInteger(pub i64);

#[derive(Default, Debug, Eq, PartialEq, Clone)]
/// The list type.
pub struct BList<'a>(pub Vec<Item<'a>>);

#[derive(Default, Debug, Eq, PartialEq, Clone)]
/// The dictionary type.
pub struct BDictionary<'a>(pub BTreeMap<BString<'a>, Item<'a>>);

impl<'a> Item<'a> {
    /// Returns a string if the current variant is a string.
    pub fn string(self) -> Option<BString<'a>> {
        match self {
            Item::String(s) => Some(s),
            _ => None,
        }
    }

    /// Returns an integer if the current variant is an integer.
    pub fn integer(self) -> Option<BInteger> {
        match self {
            Item::Integer(i) => Some(i),
            _ => None,
        }
    }

    /// Returns a list if the current variant is a list.
    pub fn list(self) -> Option<BList<'a>> {
        match self {
            Item::List(l) => Some(l),
            _ => None,
        }
    }

    /// Returns a dictionary if the current variant is a dictionary.
    pub fn dictionary(self) -> Option<BDictionary<'a>> {
        match self {
            Item::Dictionary(d) => Some(d),
            _ => None,
        }
    }
}

impl<'a> Bencode for Item<'a> {
    fn encode(self) -> Vec<u8> {
        match self {
            Item::String(s) => s.encode(),
            Item::Integer(i) => i.encode(),
            Item::List(l) => l.encode(),
            Item::Dictionary(d) => d.encode(),
        }
    }
}

impl<'a> Bencode for BString<'a> {
    fn encode(self) -> Vec<u8> {
        format!("{}:", self.0.len())
            .into_bytes()
            .into_iter()
            .chain(self.0.into_owned().into_iter())
            .collect()
    }
}

impl<'a> Bencode for BInteger {
    fn encode(self) -> Vec<u8> {
        format!("i{}e", self.0).bytes().collect()
    }
}

impl<'a> Bencode for BList<'a> {
    fn encode(self) -> Vec<u8> {
        std::iter::once(b'l')
            .chain(self.0.into_iter().flat_map(|i| i.encode()))
            .chain(std::iter::once(b'e'))
            .collect()
    }
}

impl<'a> Bencode for BDictionary<'a> {
    fn encode(self) -> Vec<u8> {
        std::iter::once(b'd')
            .chain({
                self.0
                    .into_iter()
                    .flat_map(|(k, v)| k.encode().into_iter().chain(v.encode().into_iter()))
            })
            .chain(std::iter::once(b'e'))
            .collect()
    }
}

impl<'a> From<i64> for Item<'a> {
    fn from(i: i64) -> Self {
        Self::Integer(BInteger(i))
    }
}

impl<'a> From<&'a [u8]> for Item<'a> {
    fn from(b: &'a [u8]) -> Self {
        Self::String(BString(Cow::from(b)))
    }
}

impl<'a> From<Cow<'a, [u8]>> for Item<'a> {
    fn from(b: Cow<'a, [u8]>) -> Self {
        Self::String(BString(b))
    }
}

impl<'a> From<&'a str> for Item<'a> {
    fn from(s: &'a str) -> Self {
        Self::from(s.as_bytes())
    }
}

impl<'a> From<Vec<Item<'a>>> for Item<'a> {
    fn from(v: Vec<Item<'a>>) -> Self {
        Self::List(BList(v))
    }
}

impl<'a> From<BTreeMap<BString<'a>, Item<'a>>> for Item<'a> {
    fn from(m: BTreeMap<BString<'a>, Item<'a>>) -> Self {
        Self::Dictionary(BDictionary(m))
    }
}

impl<'a> FromIterator<(BString<'a>, Item<'a>)> for Item<'a> {
    fn from_iter<T: IntoIterator<Item = (BString<'a>, Item<'a>)>>(iter: T) -> Self {
        Self::Dictionary(BDictionary(iter.into_iter().collect()))
    }
}

impl<'a> From<&'a str> for BString<'a> {
    fn from(s: &'a str) -> Self {
        BString(Cow::from(s.as_bytes()))
    }
}

impl<'a> Display for BString<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Ok(s) = str::from_utf8(&self.0) {
            write!(f, "{}", s)
        } else {
            Debug::fmt(&self.0, f)
        }
    }
}

impl<'a> Debug for BString<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "BString(")?;

        match self.0 {
            Cow::Borrowed(_) => write!(f, "borrowed(")?,
            Cow::Owned(_) => write!(f, "owned(")?,
        }

        if let Ok(s) = str::from_utf8(&self.0) {
            write!(f, "{}", s)?;
        } else {
            Debug::fmt(&self.0, f).expect("formatting error");
        }

        write!(f, "))")
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;
    use std::str::from_utf8_unchecked;

    use crate::*;

    #[test]
    fn empty_string() {
        let expected = "0:".as_bytes().to_vec();

        assert_eq!(expected, BString::from("").encode())
    }

    #[test]
    fn string() {
        let expected = "16:sixteencharslong".as_bytes().to_vec();

        assert_eq!(expected, BString::from("sixteencharslong").encode())
    }

    #[test]
    fn integer() {
        let i = 1234567890;

        let expected = format!("i{}e", i).as_bytes().to_vec();

        let actual = BInteger(i).encode();

        assert_eq!(expected, actual)
    }

    #[test]
    fn empty_list() {
        let expected = "le";

        let actual = BList(vec![]).encode();

        assert_eq!(expected, unsafe { from_utf8_unchecked(&actual) })
    }

    #[test]
    fn simple_list() {
        let expected = "li1337e5:stuffe";

        let actual = BList(vec![1337.into(), "stuff".into()]).encode();

        assert_eq!(expected, unsafe { from_utf8_unchecked(&actual) })
    }

    #[test]
    fn nested_list() {
        let expected = "llll3:fooeeee";

        let actual = BList(vec![vec![vec![vec!["foo".into()].into()].into()].into()]).encode();

        assert_eq!(expected, unsafe { from_utf8_unchecked(&actual) })
    }

    #[test]
    fn empty_dictionary() {
        let expected = "de";

        let actual = BDictionary(BTreeMap::new()).encode();

        assert_eq!(expected, unsafe { from_utf8_unchecked(&actual) })
    }

    #[test]
    fn simple_dictionary() {
        let expected = "d3:fooli34e3:bari-50eee";

        let mut map = BTreeMap::new();
        map.insert("foo".into(), vec![34.into(), "bar".into(), (-50).into()].into());

        let actual = BDictionary(map).encode();

        assert_eq!(expected, unsafe { from_utf8_unchecked(&actual) })
    }
}