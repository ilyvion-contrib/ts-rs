#![cfg(feature = "serde-compat")]
#![allow(dead_code)]

use serde::Deserialize;
use ts_rs::{Config, TS};

fn deserialize_custom<'de, D: serde::Deserializer<'de>>(d: D) -> Result<Option<i32>, D::Error> {
    Option::deserialize(d)
}

#[derive(Deserialize, TS)]
#[ts(export, export_to = "serde_deserialize_with/")]
struct DeserializeWith {
    #[serde(default, deserialize_with = "deserialize_custom")]
    a: Option<i32>,
}

#[test]
fn deserialize_with_keeps_the_field_type() {
    let cfg = Config::from_env();
    assert_eq!(
        DeserializeWith::decl(&cfg),
        "type DeserializeWith = { a: number | null, };"
    );
}
