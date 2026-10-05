use proc_macro2::{Ident, TokenStream};
use quote::quote;
use syn::{Data, DataStruct, DeriveInput, Error, Fields, Type};

/// Expands `#[derive(ShaderPermutation)]`: each field a WGSL `override` constant of its name, set from its value when
/// a pipeline is made.
pub fn expand(input: &DeriveInput) -> syn::Result<TokenStream> {
  let name: &Ident = &input.ident;
  let fields = match &input.data {
    Data::Struct(DataStruct {
      fields: Fields::Named(fields),
      ..
    }) => &fields.named,
    _ => {
      return Err(Error::new_spanned(
        name,
        "`ShaderPermutation` can only be derived for a struct with named fields",
      ));
    }
  };

  if !input.generics.params.is_empty() {
    return Err(Error::new_spanned(
      &input.generics,
      "`ShaderPermutation` cannot be derived for a generic struct",
    ));
  }

  let core: TokenStream = quote!(::xrf_renderer_core);
  let idents: Vec<&Ident> = fields
    .iter()
    .map(|field| field.ident.as_ref().expect("named fields"))
    .collect();
  let names: Vec<String> = idents.iter().map(|ident| ident.to_string()).collect();
  let types: Vec<&Type> = fields.iter().map(|field| &field.ty).collect();

  Ok(quote! {
    impl #core::ShaderPermutation for #name {
      fn list_constants(&self) -> ::std::vec::Vec<(&'static str, f64)> {
        ::std::vec![#((#names, #core::ShaderOverride::to_constant(&self.#idents))),*]
      }

      fn get_wgsl_overrides() -> ::std::string::String {
        let mut wgsl: ::std::string::String = ::std::string::String::new();

        #(wgsl.push_str(&::std::format!("override {}: {};\n", #names, <#types as #core::ShaderOverride>::WGSL_TYPE));)*
        wgsl
      }
    }
  })
}
