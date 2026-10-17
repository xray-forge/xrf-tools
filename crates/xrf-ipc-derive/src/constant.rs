use proc_macro2::TokenStream;
use quote::{format_ident, quote_spanned};
use syn::spanned::Spanned;
use syn::{Attribute, Error, Expr, ExprLit, Ident, Item, ItemConst, Lit, Meta, MetaNameValue};

/// Expands `#[constant]`: the item as it is, then a module beside it registering it as an `xrf_ipc::IpcConstant`.
pub fn expand(arguments: TokenStream, item: TokenStream) -> syn::Result<TokenStream> {
  if !arguments.is_empty() {
    return Err(Error::new_spanned(arguments, "`#[constant]` takes no arguments"));
  }

  // `syn` reads a generic `const` as verbatim tokens, so it is refused here with every other item.
  let declared: ItemConst = match syn::parse2::<Item>(item)? {
    Item::Const(declared) => declared,
    other => {
      return Err(Error::new_spanned(
        other,
        "`#[constant]` marks a module-level `const` item, whose value the frontend's bindings export",
      ));
    }
  };

  let name: &Ident = &declared.ident;
  let name_text: String = name.to_string();
  let doc: String = read_doc(&declared.attrs)?;
  // A module rather than an anonymous `const`: Rust refuses a module in an `impl` or a trait with a message naming the
  // place, where an associated constant would otherwise fail inside the generated code.
  let registration: Ident = format_ident!("__xrf_ipc_constant_{}", name);

  Ok(quote_spanned! {declared.span()=>
    #declared

    #[doc(hidden)]
    #[allow(non_snake_case)]
    mod #registration {
      ::xrf_ipc::inventory::submit! {
        ::xrf_ipc::IpcConstant {
          module_path: ::xrf_ipc::parent_module_path(::core::module_path!()),
          name: #name_text,
          doc: #doc,
          value: || ::xrf_ipc::to_value(&super::#name),
        }
      }
    }
  })
}

/// The item's doc comment: its `///` lines joined by `\n`, each without the space after its slashes, trimmed.
///
/// Refuses a doc attribute it cannot read as one such line: a block comment spanning lines, or a computed value.
fn read_doc(attrs: &[Attribute]) -> syn::Result<String> {
  let mut lines: Vec<String> = Vec::new();

  for attr in attrs.iter().filter(|attr| attr.path().is_ident("doc")) {
    let line: String = match &attr.meta {
      Meta::NameValue(MetaNameValue {
        value: Expr::Lit(ExprLit {
          lit: Lit::Str(line), ..
        }),
        ..
      }) if !line.value().contains('\n') => line.value(),
      Meta::NameValue(_) => {
        return Err(Error::new_spanned(
          attr,
          "`#[constant]` reads its doc comment from `///` lines: write this one as `///` lines",
        ));
      }
      // `#[doc(hidden)]` and the like say nothing the bindings repeat.
      _ => continue,
    };

    lines.push(line.strip_prefix(' ').map(str::to_owned).unwrap_or(line));
  }

  Ok(lines.join("\n").trim().to_owned())
}
