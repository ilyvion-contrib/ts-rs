use std::collections::HashMap;

use syn::{parse_quote, Attribute, Expr, Fields, Ident, Path, Result, Type, WherePredicate};

use super::{
    parse_assign_expr, parse_assign_from_str, parse_assign_inflection, parse_bound, parse_concrete,
    Attr, ContainerAttr, Serde, Tagged,
};
use crate::{
    attr::{parse_assign_str, parse_optional_assign_str, EnumAttr, Inflection, VariantAttr},
    optional::{parse_optional, Optional},
    utils::{extract_docs, parse_attrs},
};

#[derive(Default, Clone)]
pub struct StructAttr {
    crate_rename: Option<Path>,
    pub type_as: Option<Type>,
    pub type_override: Option<String>,
    pub rename_all: Option<Inflection>,
    pub rename: Option<Expr>,
    pub export_to: Option<Expr>,
    pub export: bool,
    pub tag: Option<String>,
    pub docs: Vec<Expr>,
    pub concrete: HashMap<Ident, Type>,
    pub bound: Option<Vec<WherePredicate>>,
    pub optional_fields: Optional,
    // whether the wire format comes from another type through #[serde(from/try_from/into)]
    pub using_serde_conversion: bool,
    // whether the struct is serialized as its single non-skipped field through #[serde(transparent)]
    pub transparent: bool,
}

impl StructAttr {
    pub fn from_attrs(attrs: &[Attribute]) -> Result<Self> {
        let mut result = parse_attrs::<Self>(attrs)?;

        if cfg!(feature = "serde-compat") {
            let serde_attr = crate::utils::parse_serde_attrs::<StructAttr>(attrs);
            result = result.merge(serde_attr.0);
        }

        result.docs = extract_docs(attrs);

        Ok(result)
    }

    pub fn from_variant(
        enum_attr: &EnumAttr,
        variant_attr: &VariantAttr,
        variant_fields: &Fields,
    ) -> Self {
        Self {
            crate_rename: Some(enum_attr.crate_rename()),
            rename: variant_attr.rename.clone(),
            rename_all: variant_attr.rename_all.or(match variant_fields {
                Fields::Named(_) => enum_attr.rename_all_fields,
                Fields::Unnamed(_) | Fields::Unit => None,
            }),
            tag: match variant_fields {
                Fields::Named(_) => match enum_attr
                    .tagged()
                    .expect("The variant attribute is known to be valid at this point")
                {
                    Tagged::Internally { tag } => Some(tag.to_owned()),
                    _ => None,
                },
                _ => None,
            },
            optional_fields: match (enum_attr.optional_fields, variant_attr.optional_fields) {
                (parent, Optional::Inherit) => parent,
                (_, child) => child,
            },

            // inline and skip are not supported on StructAttr
            ..Self::default()
        }
    }
}

impl Attr for StructAttr {
    type Item = Fields;

    fn merge(self, other: Self) -> Self {
        Self {
            crate_rename: self.crate_rename.or(other.crate_rename),
            type_as: self.type_as.or(other.type_as),
            type_override: self.type_override.or(other.type_override),
            rename: self.rename.or(other.rename),
            rename_all: self.rename_all.or(other.rename_all),
            export_to: self.export_to.or(other.export_to),
            export: self.export || other.export,
            tag: self.tag.or(other.tag),
            docs: other.docs,
            concrete: self.concrete.into_iter().chain(other.concrete).collect(),
            bound: match (self.bound, other.bound) {
                (Some(a), Some(b)) => Some(a.into_iter().chain(b).collect()),
                (Some(bound), None) | (None, Some(bound)) => Some(bound),
                (None, None) => None,
            },
            optional_fields: self.optional_fields.or(other.optional_fields),
            using_serde_conversion: self.using_serde_conversion || other.using_serde_conversion,
            transparent: self.transparent || other.transparent,
        }
    }

    fn assert_validity(&self, item: &Self::Item) -> Result<()> {
        if self.using_serde_conversion && !(self.type_as.is_some() || self.type_override.is_some())
        {
            syn_err!(
                r#"using `#[serde(from = "...")]`, `#[serde(try_from = "...")]` or `#[serde(into = "...")]` requires the use of `#[ts(as = "...")]` or `#[ts(type = "...")]`"#
            );
        }

        if self.type_override.is_some() {
            if self.type_as.is_some() {
                syn_err!("`as` is not compatible with `type`");
            }

            if self.rename_all.is_some() {
                syn_err!("`rename_all` is not compatible with `type`");
            }

            if self.tag.is_some() {
                syn_err!("`tag` is not compatible with `type`");
            }

            if let Optional::Optional { .. } = self.optional_fields {
                syn_err!("`optional_fields` is not compatible with `type`");
            }
        }

        if self.type_as.is_some() {
            if self.tag.is_some() {
                syn_err!("`tag` is not compatible with `as`");
            }

            if self.rename_all.is_some() {
                syn_err!("`rename_all` is not compatible with `as`");
            }

            if let Optional::Optional { .. } = self.optional_fields {
                syn_err!("`optional_fields` is not compatible with `as`");
            }
        }

        if self.transparent && self.tag.is_some() {
            syn_err!("`tag` is not compatible with `transparent`");
        }

        if !matches!(item, Fields::Named(_)) && self.tag.is_some() {
            syn_err!("`tag` cannot be used with unit or tuple structs");
        }

        Ok(())
    }
}

impl ContainerAttr for StructAttr {
    fn crate_rename(&self) -> Path {
        self.crate_rename
            .clone()
            .unwrap_or_else(|| parse_quote!(::ts_rs))
    }
}

impl_parse! {
    StructAttr(input, out) {
        "crate" => out.crate_rename = Some(parse_assign_from_str(input)?),
        "as" => out.type_as = Some(parse_assign_from_str(input)?),
        "type" => out.type_override = Some(parse_assign_str(input)?),
        "rename" => out.rename = Some(parse_assign_expr(input)?),
        "rename_all" => out.rename_all = Some(parse_assign_inflection(input)?),
        "tag" => out.tag = Some(parse_assign_str(input)?),
        "export" => out.export = true,
        "export_to" => out.export_to = Some(parse_assign_expr(input)?),
        "concrete" => out.concrete = parse_concrete(input)?,
        "bound" => out.bound = Some(parse_bound(input)?),
        "optional_fields" => out.optional_fields = parse_optional(input)?,
    }
}

impl_parse! {
    Serde<StructAttr>(input, out) {
        "rename" => out.0.rename = Some(parse_assign_expr(input)?),
        "rename_all" => out.0.rename_all = Some(parse_assign_inflection(input)?),
        "tag" => out.0.tag = Some(parse_assign_str(input)?),
        "bound" => out.0.bound = Some(parse_bound(input)?),
        // parse #[serde(default)] to not emit a warning
        "deny_unknown_fields" | "default" => {
            parse_optional_assign_str(input)?;
        },
        // parse #[serde(crate = "...")] to not emit a warning
        "crate" => {
            parse_assign_str(input)?;
        },
        "from" | "try_from" | "into" => {
            parse_assign_str(input)?;
            out.0.using_serde_conversion = true;
        },
        "transparent" => out.0.transparent = true,
    }
}

#[cfg(all(test, feature = "serde-compat"))]
mod tests {
    use syn::{parse_quote, ItemStruct};

    use super::*;

    fn validate(item: &ItemStruct) -> Result<()> {
        StructAttr::from_attrs(&item.attrs)?.assert_validity(&item.fields)
    }

    #[test]
    fn serde_conversions_require_a_ts_override() {
        let items: [ItemStruct; 3] = [
            parse_quote! { #[serde(from = "Wire")] struct A; },
            parse_quote! { #[serde(try_from = "Wire")] struct A; },
            parse_quote! { #[serde(into = "Wire")] struct A; },
        ];

        for item in &items {
            let error = validate(item).unwrap_err().to_string();
            assert!(error.contains("requires the use of"), "{error}");
        }
    }

    #[test]
    fn serde_conversions_are_accepted_with_a_ts_override() {
        let items: [ItemStruct; 2] = [
            parse_quote! {
                #[ts(as = "Wire")]
                #[serde(into = "Wire", try_from = "Wire")]
                struct A;
            },
            parse_quote! {
                #[ts(type = "string")]
                #[serde(from = "Wire", into = "Wire")]
                struct A;
            },
        ];

        for item in &items {
            validate(item).unwrap();
        }
    }

    #[test]
    fn transparent_is_parsed_without_a_warning() {
        let item: ItemStruct = parse_quote! { #[serde(transparent)] struct A(i32); };

        let warnings_before = crate::utils::warning::emitted();
        let attr = StructAttr::from_attrs(&item.attrs).unwrap();

        assert_eq!(crate::utils::warning::emitted(), warnings_before);
        assert!(attr.transparent);
    }

    #[test]
    fn transparent_is_incompatible_with_tag() {
        let item: ItemStruct = parse_quote! {
            #[serde(transparent, tag = "kind")]
            struct A { a: i32 }
        };

        let error = validate(&item).unwrap_err().to_string();
        assert!(
            error.contains("`tag` is not compatible with `transparent`"),
            "{error}"
        );
    }
}
