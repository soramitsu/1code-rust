use std::ops::{AddAssign, MulAssign, Neg};

use serde::de::{self, DeserializeSeed, EnumAccess, MapAccess, SeqAccess, VariantAccess, Visitor};
use serde::Deserialize;

use crate::error::{Error, Result};

pub struct Deserializer<'de> {
    input: &'de str,
}

impl<'de> Deserializer<'de> {
    pub fn from_str(input: &'de str) -> Self {
        Deserializer { input }
    }
}

pub fn from_str<'a, T>(s: &'a str) -> Result<T>
where
    T: Deserialize<'a>,
{
    let mut deserializer = Deserializer::from_str(s);
    let t = T::deserialize(&mut deserializer)?;
    if deserializer.input.is_empty() {
        Ok(t)
    } else {
        Err(Error::TrailingCharacters)
    }
}

impl<'de> Deserializer<'de> {
    /// Look at the first character in the input without consuming it.
    fn peek_char(&mut self) -> Result<char> {
        self.input.chars().next().ok_or(Error::Eof)
    }

    /// Consume the first character in the input.
    fn next_char(&mut self) -> Result<char> {
        let ch = self.peek_char()?;
        self.input = &self.input[ch.len_utf8()..];
        Ok(ch)
    }

    /// | rust  | 1coded |
    /// | ----- | ------ |
    /// | true  |   T    |
    /// | false |   F    |
    fn parse_bool(&mut self) -> Result<bool> {
        if self.input.starts_with("T") {
            self.input = &self.input["T".len()..];
            Ok(true)
        } else if self.input.starts_with("F") {
            self.input = &self.input["F".len()..];
            Ok(false)
        } else {
            Err(Error::ExpectedBoolean)
        }
    }

    /// Leading zeros aren't allowed in `1code`.
    /// | rust  | 1coded |
    /// | ----- | ------ |
    /// | 0     | i0e    |
    /// | 42    | i42e   |
    /// | -1    | i-1e   |
    /// | 1.5   | i1.5e  |
    fn parse_unsigned<T>(&mut self) -> Result<T>
    where
        T: AddAssign<T> + MulAssign<T> + From<u8>,
    {
        if self.next_char()? != 'i' {
            return Err(Error::ExpectedInteger);
        }
        let mut int = T::from(0);
        loop {
            match self.input.chars().next() {
                Some(ch @ '0'..='9') => {
                    self.input = &self.input[1..];
                    int *= T::from(10);
                    int += T::from(ch as u8 - b'0');
                }
                Some('e') => {
                    self.input = &self.input[1..];
                    return Ok(int);
                }
                _ => {
                    return Err(Error::ExpectedInteger);
                }
            }
        }
    }

    /// Parse a possible minus sign in front of an integer.
    /// Because in `1code` we have `-` sign between `i` and `e` it's easier to copy
    /// `parse_unsigned`.
    fn parse_signed<T>(&mut self) -> Result<T>
    where
        T: Neg<Output = T> + AddAssign<T> + MulAssign<T> + From<i8>,
    {
        if self.next_char()? != 'i' {
            return Err(Error::ExpectedInteger);
        }
        let mut negative = false;
        if self.peek_char()? == '-' {
            negative = true;
            self.input = &self.input[1..];
        }
        let mut int = T::from(0);
        loop {
            match self.input.chars().next() {
                Some(ch @ '0'..='9') => {
                    self.input = &self.input[1..];
                    int *= T::from(10);
                    int += T::from(ch as i8 - b'0' as i8);
                }
                Some('e') => {
                    self.input = &self.input[1..];
                    if negative {
                        return Ok(int.neg());
                    } else {
                        return Ok(int);
                    }
                }
                _ => {
                    return Err(Error::ExpectedInteger);
                }
            }
        }
    }

    /// Escape symblos are not supported.
    /// | rust  | 1coded |
    /// | ----- | ------ |
    /// | ""    | 0:     |
    /// | "0.1" | 3:0.1  |
    /// | "h h" | 3:h h  |
    fn parse_string(&mut self) -> Result<&'de str> {
        //TODO: rewrite to a logic - check that first char is int
        //self.input.find(':')
        //use result as length of length's int
        //parse length - take string
        let mut length = match self.next_char()? {
            ch @ '0'..='9' => usize::from(ch as u8 - b'0'),
            _ => {
                return Err(Error::ExpectedString);
            }
        };
        loop {
            match self.input.chars().next() {
                Some(ch @ '0'..='9') => {
                    self.input = &self.input[1..];
                    length *= usize::from(10 as u8);
                    length += usize::from(ch as u8 - b'0');
                }
                Some(':') => {
                    self.input = &self.input[1..];
                    break;
                }
                _ => {
                    return Err(Error::ExpectedString);
                }
            }
        }
        let string = &self.input[..length];
        self.input = &self.input[length..];
        return Ok(string);
    }
}

impl<'de, 'a> de::Deserializer<'de> for &'a mut Deserializer<'de> {
    type Error = Error;

    /// Because `1code` is self-describing format we can support `deserialize_any`.
    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        match self.peek_char()? {
            'N' => self.deserialize_unit(visitor),
            't' | 'f' => self.deserialize_bool(visitor),
            '0'..='9' => self.deserialize_str(visitor),
            'i' => {
                let mut iterator = self.input.chars();
                let _first_element = iterator.next().ok_or(Error::Eof)?;
                match iterator.next().ok_or(Error::Eof)? {
                    '-' => self.deserialize_i64(visitor),
                    '0'..='9' => self.deserialize_u64(visitor),
                    _ => Err(Error::Syntax),
                }
            }
            'l' => self.deserialize_seq(visitor),
            'd' => self.deserialize_map(visitor),
            _ => Err(Error::Syntax),
        }
    }

    fn deserialize_bool<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_bool(self.parse_bool()?)
    }

    fn deserialize_i8<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_i8(self.parse_signed()?)
    }

    fn deserialize_i16<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_i16(self.parse_signed()?)
    }

    fn deserialize_i32<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_i32(self.parse_signed()?)
    }

    fn deserialize_i64<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_i64(self.parse_signed()?)
    }

    fn deserialize_u8<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_u8(self.parse_unsigned()?)
    }

    fn deserialize_u16<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_u16(self.parse_unsigned()?)
    }

    fn deserialize_u32<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_u32(self.parse_unsigned()?)
    }

    fn deserialize_u64<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_u64(self.parse_unsigned()?)
    }

    fn deserialize_f32<V>(self, _visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        unimplemented!()
    }

    fn deserialize_f64<V>(self, _visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        unimplemented!()
    }

    fn deserialize_char<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let string = self.parse_string()?;
        if string.len() != 1 {
            return Err(Error::Syntax);
        } else {
            visitor.visit_char(
                string
                    .chars()
                    .next()
                    .expect("Next char should be presented."),
            )
        }
    }

    fn deserialize_str<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_borrowed_str(self.parse_string()?)
    }

    fn deserialize_string<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_str(visitor)
    }

    fn deserialize_bytes<V>(self, _visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        unimplemented!()
    }

    fn deserialize_byte_buf<V>(self, _visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        unimplemented!()
    }

    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        if self.input.starts_with("N") {
            self.input = &self.input["N".len()..];
            visitor.visit_none()
        } else {
            visitor.visit_some(self)
        }
    }

    fn deserialize_unit<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        if self.input.starts_with("N") {
            self.input = &self.input["N".len()..];
            visitor.visit_unit()
        } else {
            Err(Error::ExpectedNull)
        }
    }

    fn deserialize_unit_struct<V>(self, _name: &'static str, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_unit(visitor)
    }

    fn deserialize_newtype_struct<V>(self, _name: &'static str, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_newtype_struct(self)
    }

    fn deserialize_seq<V>(mut self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        if self.next_char()? == 'l' {
            let value = visitor.visit_seq(NotSeparated::new(&mut self))?;
            if self.next_char()? == 'e' {
                Ok(value)
            } else {
                Err(Error::ExpectedListEnd)
            }
        } else {
            Err(Error::ExpectedList)
        }
    }

    fn deserialize_tuple<V>(self, _len: usize, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_seq(visitor)
    }

    fn deserialize_tuple_struct<V>(
        self,
        _name: &'static str,
        _len: usize,
        visitor: V,
    ) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_seq(visitor)
    }

    fn deserialize_map<V>(mut self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        if self.next_char()? == 'd' {
            let value = visitor.visit_map(NotSeparated::new(&mut self))?;
            if self.next_char()? == 'e' {
                Ok(value)
            } else {
                Err(Error::ExpectedDictionaryEnd)
            }
        } else {
            Err(Error::ExpectedDictionary)
        }
    }

    fn deserialize_struct<V>(
        self,
        _name: &'static str,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_map(visitor)
    }

    fn deserialize_enum<V>(
        self,
        _name: &'static str,
        _variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        if self.next_char()? == 'd' {
            let value = visitor.visit_enum(Enum::new(self))?;
            if self.next_char()? == 'e' {
                Ok(value)
            } else {
                Err(Error::ExpectedDictionaryEnd)
            }
        } else {
            Err(Error::ExpectedDictionary)
        }
    }

    fn deserialize_identifier<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_str(visitor)
    }

    fn deserialize_ignored_any<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_any(visitor)
    }
}

struct NotSeparated<'a, 'de: 'a> {
    de: &'a mut Deserializer<'de>,
}

impl<'a, 'de> NotSeparated<'a, 'de> {
    fn new(de: &'a mut Deserializer<'de>) -> Self {
        NotSeparated { de }
    }
}

impl<'de, 'a> SeqAccess<'de> for NotSeparated<'a, 'de> {
    type Error = Error;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>>
    where
        T: DeserializeSeed<'de>,
    {
        if self.de.peek_char()? == 'e' {
            return Ok(None);
        } else {
            seed.deserialize(&mut *self.de).map(Some)
        }
    }
}

impl<'de, 'a> MapAccess<'de> for NotSeparated<'a, 'de> {
    type Error = Error;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>>
    where
        K: DeserializeSeed<'de>,
    {
        //TODO: How we should understand is it end of the map or something inside?
        //Maybe we can hold a counter inside `NotSeparated` and
        //increment it on open chars (`i`, `l`, `d`) while decrement till 0.
        //But maybe serde will parse and consume all previous `e`?
        if self.de.peek_char()? == 'e' {
            return Ok(None);
        }
        seed.deserialize(&mut *self.de).map(Some)
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value>
    where
        V: DeserializeSeed<'de>,
    {
        seed.deserialize(&mut *self.de)
    }
}

struct Enum<'a, 'de: 'a> {
    de: &'a mut Deserializer<'de>,
}

impl<'a, 'de> Enum<'a, 'de> {
    fn new(de: &'a mut Deserializer<'de>) -> Self {
        Enum { de }
    }
}

impl<'de, 'a> EnumAccess<'de> for Enum<'a, 'de> {
    type Error = Error;
    type Variant = Self;

    fn variant_seed<V>(self, seed: V) -> Result<(V::Value, Self::Variant)>
    where
        V: DeserializeSeed<'de>,
    {
        // The `deserialize_enum` method parsed a `d` character so we are
        // currently inside of a dictionary. The seed will be deserializing itself from
        // the key of the dictionary.
        let val = seed.deserialize(&mut *self.de)?;
        Ok((val, self))
    }
}

impl<'de, 'a> VariantAccess<'de> for Enum<'a, 'de> {
    type Error = Error;

    fn unit_variant(self) -> Result<()> {
        Err(Error::ExpectedString)
    }

    fn newtype_variant_seed<T>(self, seed: T) -> Result<T::Value>
    where
        T: DeserializeSeed<'de>,
    {
        seed.deserialize(self.de)
    }

    fn tuple_variant<V>(self, _len: usize, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        de::Deserializer::deserialize_seq(self.de, visitor)
    }

    fn struct_variant<V>(self, _fields: &'static [&'static str], visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        de::Deserializer::deserialize_map(self.de, visitor)
    }
}

#[test]
fn test_int() {
    let test_1code = r#"i1e"#;
    let expected = 1;
    assert_eq!(expected, from_str(test_1code).unwrap());
}

#[test]
fn test_string() {
    let test_1code = r#"5:hello"#;
    let expected = "hello".to_string();
    let actual: String = from_str(test_1code).unwrap();
    assert_eq!(expected, actual);
}

#[test]
fn test_seq() {
    let test_1code = r#"l1:a1:be"#;
    let expected: Vec<String> = vec!["a".to_owned(), "b".to_owned()];
    let actual: Vec<String> = from_str(test_1code).unwrap();
    assert_eq!(expected, actual);
}

#[test]
fn test_struct() {
    #[derive(Deserialize, PartialEq, Debug)]
    struct Test {
        int: u32,
    }
    let test_1code = r#"d3:inti1ee"#;
    let expected = Test { int: 1 };
    assert_eq!(expected, from_str(test_1code).unwrap());
}

#[test]
fn test_struct_with_seq() {
    #[derive(Deserialize, PartialEq, Debug)]
    struct Test {
        int: u32,
        seq: Vec<String>,
    }

    let test_1code = r#"d3:inti1e3:seql1:a1:bee"#;
    let expected = Test {
        int: 1,
        seq: vec!["a".to_owned(), "b".to_owned()],
    };
    assert_eq!(expected, from_str(test_1code).unwrap());
}

#[test]
//TODO: should agree on https://github.com/soramitsu/1code-java/issues/26 first
#[ignore]
fn test_enum() {
    #[derive(Deserialize, PartialEq, Debug)]
    enum E {
        Unit,
        Newtype(u32),
        Tuple(u32, u32),
        Struct { a: u32 },
    }

    let j = r#""Unit""#;
    let expected = E::Unit;
    assert_eq!(expected, from_str(j).unwrap());

    let j = r#"{"Newtype":1}"#;
    let expected = E::Newtype(1);
    assert_eq!(expected, from_str(j).unwrap());

    let j = r#"{"Tuple":[1,2]}"#;
    let expected = E::Tuple(1, 2);
    assert_eq!(expected, from_str(j).unwrap());

    let j = r#"{"Struct":{"a":1}}"#;
    let expected = E::Struct { a: 1 };
    assert_eq!(expected, from_str(j).unwrap());
}
