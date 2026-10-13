use proc_macro2::TokenStream;
use quote::quote;
use syn::{Attribute, Error, Expr, ExprLit, Item, ItemConst, Lit, Meta, MetaNameValue};

/// Expands `#[constant]`: the item as it is, then its registration as an `xrf_ipc::IpcConstant`.
pub fn expand(arguments: TokenStream, item: TokenStream) -> syn::Result<TokenStream> {
  if !arguments.is_empty() {
    return Err(Error::new_spanned(arguments, "`#[constant]` takes no arguments"));
  }

  let declared: ItemConst = match syn::parse2::<Item>(item)? {
    Item::Const(declared) => declared,
    other => {
      return Err(Error::new_spanned(
        other,
        "`#[constant]` marks a `const` item, whose value the frontend's bindings export",
      ));
    }
  };

  if !declared.generics.params.is_empty() {
    return Err(Error::new_spanned(
      &declared.generics,
      "`#[constant]` cannot export a generic constant, which has no one value",
    ));
  }

  let name = &declared.ident;
  let name_text: String = name.to_string();
  let doc: String = read_doc(&declared.attrs);

  Ok(quote! {
    #declared

    const _: () = {
      fn value() -> ::xrf_ipc::serde_json::Value {
        ::xrf_ipc::to_value(&#name)
      }

      ::xrf_ipc::inventory::submit! {
        ::xrf_ipc::IpcConstant {
          module_path: ::core::module_path!(),
          name: #name_text,
          doc: #doc,
          value,
        }
      }
    };
  })
}

/// The item's doc comment: its `///` lines joined by `\n`, each without the space after its slashes, trimmed.
fn read_doc(attrs: &[Attribute]) -> String {
  attrs
    .iter()
    .filter(|attr| attr.path().is_ident("doc"))
    .filter_map(|attr| match &attr.meta {
      Meta::NameValue(MetaNameValue {
        value: Expr::Lit(ExprLit {
          lit: Lit::Str(line), ..
        }),
        ..
      }) => Some(line.value()),
      _ => None,
    })
    .map(|line| line.strip_prefix(' ').map(str::to_owned).unwrap_or(line))
    .collect::<Vec<String>>()
    .join("\n")
    .trim()
    .to_owned()
}
