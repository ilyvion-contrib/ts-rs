use syn::{ext::IdentExt, Expr, Field, Fields, ItemStruct, Result};

use crate::{
    attr::{Attr, FieldAttr, StructAttr},
    DerivedTS,
};

mod r#enum;
mod named;
mod newtype;
mod tuple;
mod type_as;
mod type_override;
mod unit;

pub(crate) use r#enum::r#enum_def;

use crate::utils::make_string_literal;

pub(crate) fn struct_def(s: &ItemStruct) -> Result<DerivedTS> {
    let attr = StructAttr::from_attrs(&s.attrs)?;

    let ts_name = attr
        .rename
        .clone()
        .unwrap_or_else(|| make_string_literal(&s.ident.unraw().to_string(), s.ident.span()));
    type_def(&attr, ts_name, &s.fields)
}

fn type_def(attr: &StructAttr, ts_name: Expr, fields: &Fields) -> Result<DerivedTS> {
    attr.assert_validity(fields)?;

    if let Some(attr_type_override) = &attr.type_override {
        return type_override::type_override_struct(attr, ts_name, attr_type_override);
    }
    if let Some(attr_type_as) = &attr.type_as {
        return type_as::type_as_struct(attr, ts_name, attr_type_as);
    }

    if attr.transparent {
        let (inner, field_attr) = transparent_field(fields)?;
        return newtype::newtype(attr, ts_name, inner, field_attr);
    }

    match fields {
        Fields::Named(named) => match named.named.len() {
            0 if attr.tag.is_none() => Ok(unit::empty_object(attr, ts_name)),
            _ => named::named(attr, ts_name, named),
        },
        Fields::Unnamed(unnamed) => match unnamed.unnamed.len() {
            0 => Ok(unit::empty_array(attr, ts_name)),
            1 => {
                let inner = unnamed.unnamed.first().unwrap();
                newtype::newtype(attr, ts_name, inner, FieldAttr::from_attrs(&inner.attrs)?)
            }
            _ => tuple::tuple(attr, ts_name, unnamed),
        },
        Fields::Unit => Ok(unit::null(attr, ts_name)),
    }
}

/// Finds the single non-skipped field that a `#[serde(transparent)]` struct is serialized as.
fn transparent_field(fields: &Fields) -> Result<(&Field, FieldAttr)> {
    let mut found = None;
    for field in fields {
        let field_attr = FieldAttr::from_attrs(&field.attrs)?;
        if field_attr.skip {
            continue;
        }
        if found.is_some() {
            syn_err!("`transparent` requires exactly one field that is not skipped");
        }
        found = Some((field, field_attr));
    }

    match found {
        Some(found) => Ok(found),
        None => syn_err!("`transparent` requires exactly one field that is not skipped"),
    }
}
