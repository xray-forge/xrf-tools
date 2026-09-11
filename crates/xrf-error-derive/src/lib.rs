use proc_macro::TokenStream;

use convert_case::{Case, Casing};
use proc_macro2::{Ident, TokenStream as TokenStream2};
use quote::quote;
use syn::{Attribute, Data, DataEnum, DeriveInput, Error, Fields, LitStr, Meta, Variant, parse_macro_input};

#[proc_macro_derive(ErrorConstructors, attributes(constructor))]
pub fn error_constructors_derive(input: TokenStream) -> TokenStream {
  let input: DeriveInput = parse_macro_input!(input as DeriveInput);

  expand_error_constructors(&input)
    .unwrap_or_else(Error::into_compile_error)
    .into()
}

fn expand_error_constructors(input: &DeriveInput) -> syn::Result<TokenStream2> {
  let enum_name: &Ident = &input.ident;

  let variants = match &input.data {
    Data::Enum(DataEnum { variants, .. }) => variants,
    _ => {
      return Err(Error::new_spanned(
        &input.ident,
        "`ErrorConstructors` can only be derived for enums",
      ));
    }
  };

  let mut constructors: Vec<TokenStream2> = Vec::new();
  let mut errors: Option<Error> = None;

  for variant in variants {
    match expand_constructor(variant) {
      Ok(Some(constructor)) => constructors.push(constructor),
      Ok(None) => {}
      Err(error) => match &mut errors {
        Some(errors) => errors.combine(error),
        None => errors = Some(error),
      },
    }
  }

  if let Some(errors) = errors {
    return Err(errors);
  }

  Ok(quote! {
      impl #enum_name {
          #(#constructors)*
      }
  })
}

fn expand_constructor(variant: &Variant) -> syn::Result<Option<TokenStream2>> {
  let attributes: Vec<&Attribute> = variant
    .attrs
    .iter()
    .filter(|attribute| attribute.path().is_ident("constructor"))
    .collect();

  let Some(attribute) = attributes.first() else {
    return Ok(None);
  };

  if let Some(duplicate) = attributes.get(1) {
    return Err(Error::new_spanned(duplicate, "duplicate #[constructor] attribute"));
  }

  validate_constructor_variant(variant)?;

  let function_name: Ident = match &attribute.meta {
    Meta::Path(_) => Ident::new(
      &format!("new_{}_error", variant.ident.to_string().to_case(Case::Snake)),
      variant.ident.span(),
    ),
    Meta::List(_) => {
      let literal: LitStr = attribute.parse_args()?;
      syn::parse_str::<Ident>(&literal.value())
        .map_err(|_| Error::new(literal.span(), "constructor name must be a valid Rust identifier"))?
    }
    Meta::NameValue(_) => {
      return Err(Error::new_spanned(
        attribute,
        "expected #[constructor] or #[constructor(\"name\")]",
      ));
    }
  };

  let variant_name: &Ident = &variant.ident;

  Ok(Some(quote! {
          pub fn #function_name<T>(message: T) -> Self
          where
              T: Into<String>,
          {
              Self::#variant_name { message: message.into() }
          }
  }))
}

fn validate_constructor_variant(variant: &Variant) -> syn::Result<()> {
  let Fields::Named(fields) = &variant.fields else {
    return Err(Error::new_spanned(
      &variant.ident,
      "#[constructor] requires exactly one named `message` field",
    ));
  };

  let is_message_only: bool = fields.named.len() == 1
    && fields
      .named
      .first()
      .and_then(|field| field.ident.as_ref())
      .is_some_and(|name| name == "message");

  if !is_message_only {
    return Err(Error::new_spanned(
      fields,
      "#[constructor] requires exactly one named `message` field",
    ));
  }

  Ok(())
}
