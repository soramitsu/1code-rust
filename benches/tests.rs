#[macro_use]
extern crate bencher;

use bencher::Bencher;
use onecode::ser as one_code_ser;

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

fn serialize(bench: &mut Bencher) {
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
    bench.iter(|| {
        one_code_ser::to_string(&struct_to_serialize).unwrap();
    });
}

benchmark_group!(benches, serialize);
benchmark_main!(benches);
