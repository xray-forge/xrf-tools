use proc_macro2::{Ident, TokenStream};
use quote::quote;
use syn::ext::IdentExt;
use syn::punctuated::Punctuated;
use syn::{
  Attribute, Data, DataEnum, DataStruct, DeriveInput, Error, Expr, Field, Fields, LitStr, Token, Type, TypePath,
};

/// Expands `#[derive(RenderSettingSchema)]`: a struct describes each field it does not skip under its serialized name,
/// a unit enum is a choice of its variants as they serialize.
pub fn expand(input: &DeriveInput) -> syn::Result<TokenStream> {
  let name: &Ident = &input.ident;

  if !input.generics.params.is_empty() {
    return Err(Error::new_spanned(
      &input.generics,
      "`RenderSettingSchema` cannot be derived for a generic type",
    ));
  }

  let schema: TokenStream = quote!(crate::contract::settings::schema);
  let body: TokenStream = match &input.data {
    Data::Struct(DataStruct {
      fields: Fields::Named(fields),
      ..
    }) => {
      let is_camel_case: bool = match read_serde_string(&input.attrs, "rename_all")? {
        None => false,
        Some(rule) if rule.value() == "camelCase" => true,
        Some(rule) => {
          return Err(Error::new_spanned(
            rule,
            "`RenderSettingSchema` reads a struct's fields as written or under `rename_all = \"camelCase\"`",
          ));
        }
      };
      let entries: Vec<TokenStream> = fields
        .named
        .iter()
        .map(|field| describe_field(field, is_camel_case, &schema))
        .collect::<syn::Result<Vec<Option<TokenStream>>>>()?
        .into_iter()
        .flatten()
        .collect();

      quote! {
        let mut group: ::std::collections::BTreeMap<::std::string::String, #schema::render_setting_node::RenderSettingNode> =
          ::std::collections::BTreeMap::new();

        #(#entries)*
        #schema::render_setting_node::RenderSettingNode::Group(group)
      }
    }
    Data::Enum(choices) => {
      let values: Vec<String> = list_choices(input, choices)?;

      quote! {
        #schema::render_setting_node::RenderSettingNode::Field(#schema::render_setting_field::RenderSettingField::Choice {
          values: ::std::vec![#(::std::string::String::from(#values)),*],
        })
      }
    }
    _ => {
      return Err(Error::new_spanned(
        name,
        "`RenderSettingSchema` can only be derived for a struct with named fields or an enum of unit variants",
      ));
    }
  };

  Ok(quote! {
    impl #schema::render_setting_schema::RenderSettingSchema for #name {
      fn describe() -> #schema::render_setting_node::RenderSettingNode {
        #body
      }
    }
  })
}

/// How one field is described, read from its `#[setting(..)]`.
enum Described {
  /// Not a setting: set some other way than by the settings.
  Skipped,
  /// A number between bounds.
  Number(Box<NumberBounds>),
  /// A run of widths between bounds, as many as the most given.
  Widths(Box<WidthsBounds>),
  /// By its type's own description: a flag, a choice or a group.
  ByType,
}

/// A run of widths' bounds and the most it holds, as its `#[setting(..)]` gives them.
struct WidthsBounds {
  min: Expr,
  max: Expr,
  most: Expr,
}

/// A number's bounds as its `#[setting(..)]` gives them, each an expression cast to `f32` where expanded.
struct NumberBounds {
  min: Expr,
  max: Expr,
  is_integer: bool,
  recommended: Option<(Expr, Expr)>,
  wasted_above: Option<Expr>,
}

/// The insertion of one field's description into the struct's group under its serialized name, or none for a field
/// skipped.
fn describe_field(field: &Field, is_camel_case: bool, schema: &TokenStream) -> syn::Result<Option<TokenStream>> {
  let ident: &Ident = field.ident.as_ref().expect("named fields");
  let key: String = match read_serde_string(&field.attrs, "rename")? {
    Some(renamed) => renamed.value(),
    None if is_camel_case => to_camel_case(&ident.unraw().to_string()),
    None => ident.unraw().to_string(),
  };
  let ty: &Type = &field.ty;

  let node: TokenStream = match read_described(field)? {
    Described::Skipped => return Ok(None),
    Described::Number(bounds) => {
      let NumberBounds {
        min,
        max,
        is_integer,
        recommended,
        wasted_above,
      } = *bounds;
      let is_integer: bool = is_integer || is_integer_type(ty);
      let recommended: TokenStream = match recommended {
        Some((low, high)) => quote!(::std::option::Option::Some([(#low) as f32, (#high) as f32])),
        None => quote!(::std::option::Option::None),
      };
      let wasted_above: TokenStream = match wasted_above {
        Some(value) => quote!(::std::option::Option::Some((#value) as f32)),
        None => quote!(::std::option::Option::None),
      };

      quote! {
        #schema::render_setting_node::RenderSettingNode::Field(#schema::render_setting_field::RenderSettingField::Number(#schema::render_setting_field::RenderSettingNumber {
          min: (#min) as f32,
          max: (#max) as f32,
          is_integer: #is_integer,
          recommended: #recommended,
          wasted_above: #wasted_above,
        }))
      }
    }
    Described::Widths(bounds) => {
      let WidthsBounds { min, max, most } = *bounds;

      quote! {
      #schema::render_setting_node::RenderSettingNode::Field(#schema::render_setting_field::RenderSettingField::Widths {
        min: (#min) as f32,
        max: (#max) as f32,
        most: (#most) as u32,
      })
      }
    }
    Described::ByType => quote!(<#ty as #schema::render_setting_schema::RenderSettingSchema>::describe()),
  };

  Ok(Some(quote! {
    group.insert(::std::string::String::from(#key), #node);
  }))
}

/// A field's `#[setting(..)]`: `skip`; `min = .., max = ..` with `integer`, `recommended = [.., ..]` and
/// `wasted_above = ..`; or `widths, min = .., max = .., most = ..`. A field without one is described by its type.
fn read_described(field: &Field) -> syn::Result<Described> {
  let attrs: Vec<&Attribute> = field
    .attrs
    .iter()
    .filter(|attr| attr.path().is_ident("setting"))
    .collect();

  let attr: &Attribute = match attrs.as_slice() {
    [] => return Ok(Described::ByType),
    [attr] => attr,
    [_, extra, ..] => return Err(Error::new_spanned(extra, "a field takes one `#[setting(..)]`")),
  };

  let (mut is_skipped, mut is_widths, mut is_integer) = (false, false, false);
  let (mut min, mut max, mut most, mut wasted_above): (Option<Expr>, Option<Expr>, Option<Expr>, Option<Expr>) =
    (None, None, None, None);
  let mut recommended: Option<(Expr, Expr)> = None;

  attr.parse_nested_meta(|meta| {
    if meta.path.is_ident("skip") {
      is_skipped = true;
    } else if meta.path.is_ident("widths") {
      is_widths = true;
    } else if meta.path.is_ident("integer") {
      is_integer = true;
    } else if meta.path.is_ident("min") {
      min = Some(meta.value()?.parse()?);
    } else if meta.path.is_ident("max") {
      max = Some(meta.value()?.parse()?);
    } else if meta.path.is_ident("most") {
      most = Some(meta.value()?.parse()?);
    } else if meta.path.is_ident("wasted_above") {
      wasted_above = Some(meta.value()?.parse()?);
    } else if meta.path.is_ident("recommended") {
      let value: syn::parse::ParseStream<'_> = meta.value()?;
      let range: syn::parse::ParseBuffer<'_>;

      syn::bracketed!(range in value);
      let bounds: Vec<Expr> = Punctuated::<Expr, Token![,]>::parse_terminated(&range)?
        .into_iter()
        .collect();
      let [low, high]: [Expr; 2] = bounds
        .try_into()
        .map_err(|_| meta.error("`recommended` is a range: `[low, high]`"))?;

      recommended = Some((low, high));
    } else {
      return Err(meta.error(
        "expected `skip`, `widths`, `integer`, `min = ..`, `max = ..`, `most = ..`, `recommended = [.., ..]` or \
         `wasted_above = ..`",
      ));
    }

    Ok(())
  })?;

  if is_skipped {
    return Ok(Described::Skipped);
  }

  let (Some(min), Some(max)) = (min, max) else {
    return Err(Error::new_spanned(
      attr,
      "a number or a run of widths is given its `min = ..` and `max = ..`",
    ));
  };

  if is_widths {
    let Some(most) = most else {
      return Err(Error::new_spanned(
        attr,
        "a run of widths is given the `most = ..` it holds",
      ));
    };

    return Ok(Described::Widths(Box::new(WidthsBounds { min, max, most })));
  }

  Ok(Described::Number(Box::new(NumberBounds {
    min,
    max,
    is_integer,
    recommended,
    wasted_above,
  })))
}

/// Each unit variant's name as serde writes it: its `#[serde(rename = "..")]`, or the enum's `rename_all` applied.
fn list_choices(input: &DeriveInput, choices: &DataEnum) -> syn::Result<Vec<String>> {
  let rule: Option<LitStr> = read_serde_string(&input.attrs, "rename_all")?;

  choices
    .variants
    .iter()
    .map(|variant| {
      if !matches!(variant.fields, Fields::Unit) {
        return Err(Error::new_spanned(variant, "a choice's variants carry no data"));
      }

      if let Some(renamed) = read_serde_string(&variant.attrs, "rename")? {
        return Ok(renamed.value());
      }

      let name: String = variant.ident.unraw().to_string();

      match &rule {
        None => Ok(name),
        Some(rule) => apply_rename_rule(rule, &name),
      }
    })
    .collect()
}

/// The string a `#[serde(key = "..")]` among `attrs` gives, ignoring serde's other keys.
fn read_serde_string(attrs: &[Attribute], key: &str) -> syn::Result<Option<LitStr>> {
  let mut found: Option<LitStr> = None;

  for attr in attrs.iter().filter(|attr| attr.path().is_ident("serde")) {
    attr.parse_nested_meta(|meta| {
      if meta.path.is_ident(key) {
        found = Some(meta.value()?.parse()?);
      } else if meta.input.peek(syn::Token![=]) {
        // Another key's value, read past so the next key parses.
        meta.value()?.parse::<Expr>()?;
      } else if meta.input.peek(syn::token::Paren) {
        let _nested: syn::parse::ParseBuffer<'_>;

        syn::parenthesized!(_nested in meta.input);
      }

      Ok(())
    })?;
  }

  Ok(found)
}

/// A variant's name, written in Rust's `PascalCase`, under the `rename_all` rule serde applies.
fn apply_rename_rule(rule: &LitStr, name: &str) -> syn::Result<String> {
  let words: Vec<String> = split_pascal_case(name);

  Ok(match rule.value().as_str() {
    "camelCase" => to_camel_case(&words.join("_")),
    "snake_case" => words.join("_"),
    "lowercase" => words.concat(),
    "kebab-case" => words.join("-"),
    "SCREAMING_SNAKE_CASE" => words.join("_").to_uppercase(),
    "PascalCase" => name.to_owned(),
    _ => {
      return Err(Error::new_spanned(
        rule,
        "`RenderSettingSchema` does not read this `rename_all` rule",
      ));
    }
  })
}

/// `RayReconstructed` as `["ray", "reconstructed"]`, a digit kept with the word before it.
fn split_pascal_case(name: &str) -> Vec<String> {
  let mut words: Vec<String> = Vec::new();

  for character in name.chars() {
    match words.last_mut() {
      Some(word) if !character.is_uppercase() => word.push(character),
      _ => words.push(character.to_lowercase().collect()),
    }
  }

  words
}

/// `motion_limit` as `motionLimit`.
fn to_camel_case(snake: &str) -> String {
  let mut camel: String = String::with_capacity(snake.len());
  let mut is_upper: bool = false;

  for character in snake.chars() {
    if character == '_' {
      is_upper = !camel.is_empty();
    } else if is_upper {
      camel.extend(character.to_uppercase());
      is_upper = false;
    } else {
      camel.push(character);
    }
  }

  camel
}

/// Whether a field's type is a whole number, which a setting of it takes whole.
fn is_integer_type(ty: &Type) -> bool {
  let Type::Path(TypePath { qself: None, path, .. }) = ty else {
    return false;
  };

  path.segments.last().is_some_and(|segment| {
    matches!(
      segment.ident.to_string().as_str(),
      "u8" | "u16" | "u32" | "u64" | "usize" | "i8" | "i16" | "i32" | "i64" | "isize"
    )
  })
}

#[cfg(test)]
mod tests {
  use syn::LitStr;

  use super::{apply_rename_rule, to_camel_case};

  #[test]
  fn names_a_field_as_serde_writes_it_in_camel_case() {
    assert_eq!(to_camel_case("motion_limit"), "motionLimit");
    assert_eq!(to_camel_case("ssa_glod_start"), "ssaGlodStart");
    assert_eq!(to_camel_case("is_enabled"), "isEnabled");
  }

  #[test]
  fn names_a_variant_under_its_enum_s_rename_rule() {
    let rule = |rule: &str| LitStr::new(rule, proc_macro2::Span::call_site());

    assert_eq!(
      apply_rename_rule(&rule("camelCase"), "RayReconstructed").unwrap(),
      "rayReconstructed"
    );
    assert_eq!(apply_rename_rule(&rule("lowercase"), "Dlss").unwrap(), "dlss");
    assert_eq!(
      apply_rename_rule(&rule("snake_case"), "Fsr2Quality").unwrap(),
      "fsr2_quality"
    );
    assert_eq!(
      apply_rename_rule(&rule("SCREAMING_SNAKE_CASE"), "Engine").unwrap(),
      "ENGINE"
    );
  }
}
