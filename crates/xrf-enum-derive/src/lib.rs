#![doc = include_str!("../README.md")]

use proc_macro::TokenStream;
use proc_macro2::{Ident, TokenStream as TokenStream2};
use quote::quote;
use syn::{Data, DataEnum, DeriveInput, Error, Fields, LitStr, Variant, parse_macro_input};

/// Implements `Display`, writing each variant's `#[display("text")]`, or its name; see the crate's README.
#[proc_macro_derive(EnumDisplay, attributes(display))]
pub fn enum_display_derive(input: TokenStream) -> TokenStream {
  expand(input, "EnumDisplay", |input, variants| {
    let (names, texts) = split(variants);
    let enum_name: &Ident = &input.ident;

    quote! {
        impl ::core::fmt::Display for #enum_name {
            fn fmt(&self, formatter: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                formatter.write_str(match self {
                    #(Self::#names => #texts,)*
                })
            }
        }
    }
  })
}

/// Implements `FromStr`, reading back the text `EnumDisplay` writes, in any ASCII case; see the crate's README.
#[proc_macro_derive(EnumFromStr, attributes(display))]
pub fn enum_from_str_derive(input: TokenStream) -> TokenStream {
  expand(input, "EnumFromStr", |input, variants| {
    let (names, texts) = split(variants);
    let enum_name: &Ident = &input.ident;
    let unknown: String = format!("Invalid `{enum_name}` string representation: '{{}}'");

    quote! {
        impl ::core::str::FromStr for #enum_name {
            type Err = ::xrf_error::XrfError;

            fn from_str(text: &str) -> ::core::result::Result<Self, Self::Err> {
                #(if text.eq_ignore_ascii_case(#texts) {
                    return ::core::result::Result::Ok(Self::#names);
                })*

                ::core::result::Result::Err(::xrf_error::XrfError::new_parsing_error(::std::format!(#unknown, text)))
            }
        }
    }
  })
}

/// Adds `const VARIANTS: &[Self]`, every variant in declaration order; see the crate's README.
#[proc_macro_derive(EnumVariants)]
pub fn enum_variants_derive(input: TokenStream) -> TokenStream {
  expand(input, "EnumVariants", |input, variants| {
    let (names, _) = split(variants);
    let enum_name: &Ident = &input.ident;

    quote! {
        impl #enum_name {
            /// Every variant, in declaration order.
            pub const VARIANTS: &'static [Self] = &[#(Self::#names),*];
        }
    }
  })
}

/// One fieldless variant and the text it reads and writes as.
struct TextVariant {
  name: Ident,
  text: String,
}

/// Parses `input` as a fieldless enum and hands its variants to `generate`, or answers the compile errors found.
fn expand(
  input: TokenStream,
  derive: &str,
  generate: impl FnOnce(&DeriveInput, &[TextVariant]) -> TokenStream2,
) -> TokenStream {
  let input: DeriveInput = parse_macro_input!(input as DeriveInput);

  read_variants(&input, derive)
    .map(|variants| generate(&input, &variants))
    .unwrap_or_else(Error::into_compile_error)
    .into()
}

fn read_variants(input: &DeriveInput, derive: &str) -> syn::Result<Vec<TextVariant>> {
  let Data::Enum(DataEnum { variants, .. }) = &input.data else {
    return Err(Error::new_spanned(
      &input.ident,
      format!("`{derive}` can only be derived for enums"),
    ));
  };

  if !input.generics.params.is_empty() {
    return Err(Error::new_spanned(
      &input.generics,
      format!("`{derive}` does not support generic enums"),
    ));
  }

  let mut read: Vec<TextVariant> = Vec::with_capacity(variants.len());
  let mut errors: Option<Error> = None;

  for variant in variants {
    match read_variant(variant) {
      Ok(variant) => read.push(variant),
      Err(error) => match &mut errors {
        Some(errors) => errors.combine(error),
        None => errors = Some(error),
      },
    }
  }

  match errors {
    Some(errors) => Err(errors),
    None => Ok(read),
  }
}

fn read_variant(variant: &Variant) -> syn::Result<TextVariant> {
  if !matches!(variant.fields, Fields::Unit) {
    return Err(Error::new_spanned(
      &variant.fields,
      "only fieldless variants are supported",
    ));
  }

  let mut texts = variant
    .attrs
    .iter()
    .filter(|attribute| attribute.path().is_ident("display"));
  let text: String = match texts.next() {
    Some(attribute) => attribute.parse_args::<LitStr>()?.value(),
    None => variant.ident.to_string(),
  };

  if let Some(duplicate) = texts.next() {
    return Err(Error::new_spanned(duplicate, "duplicate #[display] attribute"));
  }

  Ok(TextVariant {
    name: variant.ident.clone(),
    text,
  })
}

fn split(variants: &[TextVariant]) -> (Vec<&Ident>, Vec<&str>) {
  variants
    .iter()
    .map(|variant| (&variant.name, variant.text.as_str()))
    .unzip()
}
