use crate::error::{Error, Result};
use serde::{ser, Serialize};

pub struct Serializer {
    output: String,
}

pub fn to_string<T>(value: &T) -> Result<String>
where
    T: Serialize,
{
    let mut serializer = Serializer {
        output: String::new(),
    };
    value.serialize(&mut serializer)?;
    Ok(serializer.output)
}

impl<'a> ser::Serializer for &'a mut Serializer {
    type Ok = ();
    type Error = Error;
    type SerializeSeq = Self;
    type SerializeTuple = Self;
    type SerializeTupleStruct = Self;
    type SerializeTupleVariant = Self;
    type SerializeMap = Self;
    type SerializeStruct = Self;
    type SerializeStructVariant = Self;

    /// | rust  | 1coded |
    /// | ----- | ------ |
    /// | true  |   T    |
    /// | false |   F    |
    fn serialize_bool(self, value: bool) -> Result<()> {
        self.output += if value { "T" } else { "F" };
        Ok(())
    }

    /// | rust  | 1coded |
    /// | ----- | ------ |
    /// | 0     | i0e    |
    /// | 42    | i42e   |
    /// | -1    | i-1e   |
    /// | 1.5   | i1.5e  |
    fn serialize_i64(self, value: i64) -> Result<()> {
        //TODO: replace with usage of https://crates.io/crates/itoa
        self.output += &format!("i{}e", value.to_string());
        Ok(())
    }

    fn serialize_i32(self, value: i32) -> Result<()> {
        self.serialize_i64(i64::from(value))
    }

    fn serialize_i16(self, value: i16) -> Result<()> {
        self.serialize_i64(i64::from(value))
    }

    fn serialize_i8(self, value: i8) -> Result<()> {
        self.serialize_i64(i64::from(value))
    }

    /// | rust  | 1coded |
    /// | ----- | ------ |
    /// | 0     | i0e    |
    /// | 42    | i42e   |
    /// | -1    | i-1e   |
    /// | 1.5   | i1.5e  |
    fn serialize_u64(self, value: u64) -> Result<()> {
        //TODO: replace with usage of https://crates.io/crates/itoa
        self.output += &format!("i{}e", value.to_string());
        Ok(())
    }

    fn serialize_u32(self, value: u32) -> Result<()> {
        self.serialize_u64(u64::from(value))
    }

    fn serialize_u16(self, value: u16) -> Result<()> {
        self.serialize_u64(u64::from(value))
    }

    fn serialize_u8(self, value: u8) -> Result<()> {
        self.serialize_u64(u64::from(value))
    }

    /// | rust  | 1coded |
    /// | ----- | ------ |
    /// | 0     | i0e    |
    /// | 42    | i42e   |
    /// | -1    | i-1e   |
    /// | 1.5   | i1.5e  |
    fn serialize_f64(self, value: f64) -> Result<()> {
        //TODO: replace with usage of https://crates.io/crates/itoa
        self.output += &format!("i{}e", value.to_string());
        Ok(())
    }

    fn serialize_f32(self, value: f32) -> Result<()> {
        self.serialize_f64(f64::from(value))
    }

    /// | rust  | 1coded |
    /// | ----- | ------ |
    /// | ""    | 0:     |
    /// | "0.1" | 3:0.1  |
    /// | "h h" | 3:h h  |
    fn serialize_str(self, value: &str) -> Result<()> {
        self.output += &format!("{}:{}", value.len(), value);
        Ok(())
    }

    fn serialize_char(self, value: char) -> Result<()> {
        self.serialize_str(&value.to_string())
    }

    /// |    rust     |    1coded   |
    /// | ----------- | ----------- |
    /// | vec![1,2,3] | li1ei2ei3ee |
    fn serialize_bytes(self, value: &[u8]) -> Result<()> {
        use serde::ser::SerializeSeq;
        let mut seq = self.serialize_seq(Some(value.len()))?;
        for byte in value {
            seq.serialize_element(byte)?;
        }
        seq.end()
    }

    /// List serialization: start.
    fn serialize_seq(self, _length: Option<usize>) -> Result<Self::SerializeSeq> {
        self.output += "l";
        Ok(self)
    }

    fn serialize_tuple(self, length: usize) -> Result<Self::SerializeTuple> {
        self.serialize_seq(Some(length))
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        length: usize,
    ) -> Result<Self::SerializeTupleStruct> {
        self.serialize_seq(Some(length))
    }

    /// |       rust       |     1coded   |
    /// | ---------------- | ------------ |
    /// | { "str": "str" } | d3:str3:stre |
    fn serialize_newtype_variant<T>(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        value: &T,
    ) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        self.output += "d";
        variant.serialize(&mut *self)?;
        value.serialize(&mut *self)?;
        self.output += "e";
        Ok(())
    }

    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap> {
        self.output += "d";
        Ok(self)
    }

    fn serialize_struct(self, _name: &'static str, len: usize) -> Result<Self::SerializeStruct> {
        self.serialize_map(Some(len))
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant> {
        self.output += "d";
        variant.serialize(&mut *self)?;
        self.output += "d";
        Ok(self)
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant> {
        self.output += "d";
        variant.serialize(&mut *self)?;
        self.output += "l";
        Ok(self)
    }

    /// |     rust     | 1coded |
    /// | ------------ | ------ |
    /// | ()           | N      |
    /// | Option::None | N      |
    fn serialize_unit(self) -> Result<()> {
        self.output += "N";
        Ok(())
    }

    fn serialize_none(self) -> Result<()> {
        self.serialize_unit()
    }

    fn serialize_some<T>(self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(self)
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<()> {
        self.serialize_unit()
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
    ) -> Result<()> {
        //TODO decide https://github.com/soramitsu/1code-java/issues/26
        self.serialize_str(variant)
    }

    fn serialize_newtype_struct<T>(self, _name: &'static str, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(self)
    }
}

impl<'a> ser::SerializeSeq for &'a mut Serializer {
    type Ok = ();
    type Error = Error;

    fn serialize_element<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<()> {
        self.output += "e";
        Ok(())
    }
}

impl<'a> ser::SerializeTuple for &'a mut Serializer {
    type Ok = ();
    type Error = Error;

    fn serialize_element<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<()> {
        self.output += "e";
        Ok(())
    }
}

impl<'a> ser::SerializeTupleStruct for &'a mut Serializer {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<()> {
        self.output += "e";
        Ok(())
    }
}

impl<'a> ser::SerializeTupleVariant for &'a mut Serializer {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<()> {
        self.output += "ee";
        Ok(())
    }
}

impl<'a> ser::SerializeMap for &'a mut Serializer {
    type Ok = ();
    type Error = Error;

    fn serialize_key<T>(&mut self, key: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        key.serialize(&mut **self)
    }

    fn serialize_value<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<()> {
        self.output += "e";
        Ok(())
    }
}

impl<'a> ser::SerializeStruct for &'a mut Serializer {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        key.serialize(&mut **self)?;
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<()> {
        self.output += "e";
        Ok(())
    }
}

impl<'a> ser::SerializeStructVariant for &'a mut Serializer {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        key.serialize(&mut **self)?;
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<()> {
        self.output += "ee";
        Ok(())
    }
}

use std::collections::HashMap;
#[derive(serde::Serialize)]
struct StructToSerialize {
    boolean: bool,
    positive_integer: u8,
    negative_integer: i16,
    positive_float: f32,
    negative_float: f32,
    negative_float_comma: f32,
    empty_string: String,
    number_string: String,
    latin_string: String,
    cyrillic_string: String,
    japan_string: String,
    number_list: Vec<u16>,
    string_list: Vec<String>,
    number_dictionary: HashMap<String, u16>,
    string_dictionary: HashMap<String, String>,
    list_dictionary: HashMap<String, Vec<u16>>,
    null: Option<()>,
}

// Transforms struct into
/*
    d
    7:booleanT
    16:positive_integeri1e
    16:negative_integeri-1e
    14:positive_floati1.5e
    14:negative_floati-1.5e
    20:negative_float_commai-1.5e
    12:empty_string0:
    13:number_string3:0.1
    12:latin_string11:hello world
    15:cyrillic_string19:привет мир
    12:japan_string21:こんにちは世界
    11:number_listli1ei2ei3ee
    11:string_listl1:12:013:011e
    17:number_dictionaryd1:1i1ee
    17:string_dictionaryd1:11:1e
    15:list_dictionaryd1:1li1ei2ei3eee
    4:nullN
    e
*/
#[test]
fn it_works() {
    let mut number_dictionary = HashMap::new();
    number_dictionary.insert("1".to_string(), 1);
    let mut string_dictionary = HashMap::new();
    string_dictionary.insert("1".to_string(), "1".to_string());
    let mut list_dictionary = HashMap::new();
    list_dictionary.insert("1".to_string(), vec![1, 2, 3]);
    let struct_to_serialize = StructToSerialize {
        boolean: true,
        positive_integer: 1,
        negative_integer: -1,
        positive_float: 1.5,
        negative_float: -1.5,
        negative_float_comma: -1.5,
        empty_string: "".to_string(),
        number_string: "0.1".to_string(),
        latin_string: "hello world".to_string(),
        cyrillic_string: "привет мир".to_string(),
        japan_string: "こんにちは世界".to_string(),
        number_list: vec![1, 2, 3],
        string_list: vec!["1".to_string(), "01".to_string(), "011".to_string()],
        number_dictionary: number_dictionary,
        string_dictionary: string_dictionary,
        list_dictionary: list_dictionary,
        null: Option::None,
    };
    let expected_result = "d7:booleanT16:positive_integeri1e16:negative_integeri-1e14:positive_floati1.5e14:negative_floati-1.5e20:negative_float_commai-1.5e12:empty_string0:13:number_string3:0.112:latin_string11:hello world15:cyrillic_string19:привет мир12:japan_string21:こんにちは世界11:number_listli1ei2ei3ee11:string_listl1:12:013:011e17:number_dictionaryd1:1i1ee17:string_dictionaryd1:11:1e15:list_dictionaryd1:1li1ei2ei3eee4:nullNe";
    assert_eq!(to_string(&struct_to_serialize).unwrap(), expected_result);
}
