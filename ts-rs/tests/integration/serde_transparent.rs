#![cfg(feature = "serde-compat")]
#![allow(dead_code)]

use serde::Serialize;
use ts_rs::{Config, TS};

#[derive(Serialize, TS)]
#[ts(export, export_to = "serde_transparent/")]
#[serde(transparent)]
struct Newtype(u16);

#[derive(Serialize, TS)]
#[ts(export, export_to = "serde_transparent/")]
#[serde(transparent)]
struct NamedWrapper {
    inner: Vec<String>,
}

#[derive(Serialize, TS)]
#[ts(export, export_to = "serde_transparent/")]
#[serde(transparent)]
struct WrapperWithSkippedField {
    #[serde(skip)]
    cache: Option<u8>,
    value: Newtype,
}

#[test]
fn transparent_newtype_is_its_field_type() {
    let cfg = Config::from_env();
    assert_eq!(Newtype::decl(&cfg), "type Newtype = number;");
}

#[test]
fn transparent_named_struct_is_its_field_type() {
    let cfg = Config::from_env();
    assert_eq!(
        NamedWrapper::decl(&cfg),
        "type NamedWrapper = Array<string>;"
    );
}

#[test]
fn transparent_struct_ignores_skipped_fields() {
    let cfg = Config::from_env();
    assert_eq!(
        WrapperWithSkippedField::decl(&cfg),
        "type WrapperWithSkippedField = Newtype;"
    );
}
