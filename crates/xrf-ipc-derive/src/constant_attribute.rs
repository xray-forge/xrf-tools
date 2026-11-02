use proc_macro2::TokenStream;
use quote::{format_ident, quote, quote_spanned};
use syn::spanned::Spanned;
use syn::{
  Attribute, Error, Expr, ExprLit, Ident, Item, ItemConst, ItemFn, Lit, Meta, MetaNameValue, ReturnType, Safety,
  Signature,
};

/// `#[constant]`: the item as it is, then a module beside it registering it as an `xrf_ipc::IpcConstant`.
pub(crate) struct ConstantAttribute;

impl ConstantAttribute {
  /// Expands the attribute over `item`, refusing any arguments and anything but a module-level `const` or a function
  /// computing one.
  pub fn expand(arguments: TokenStream, item: TokenStream) -> syn::Result<TokenStream> {
    if !arguments.is_empty() {
      return Err(Error::new_spanned(arguments, "`#[constant]` takes no arguments"));
    }

    // `syn` reads a generic `const` as verbatim tokens, so it is refused here with every other item.
    match syn::parse2::<Item>(item)? {
      Item::Const(declared) => Self::expand_const(declared),
      Item::Fn(declared) => Self::expand_fn(declared),
      other => Err(Error::new_spanned(
        other,
        "`#[constant]` marks a module-level `const` item, or a function of no arguments computing one, whose value the \
         frontend's bindings export",
      )),
    }
  }

  /// A `const`, exported under its own name.
  fn expand_const(declared: ItemConst) -> syn::Result<TokenStream> {
    let name: &Ident = &declared.ident;
    let doc: String = Self::read_doc(&declared.attrs)?;

    Ok(Self::register(
      &declared,
      name,
      &name.to_string(),
      &doc,
      quote!(super::#name),
    ))
  }

  /// A function of no arguments, its value computed as the bindings are generated, exported under its name in upper
  /// snake case: what Rust cannot write as a `const`, such as a value holding a `Vec` or a `String`.
  fn expand_fn(declared: ItemFn) -> syn::Result<TokenStream> {
    let signature: &Signature = &declared.sig;

    if !signature.inputs.is_empty()
      || !signature.generics.params.is_empty()
      || signature.asyncness.is_some()
      || matches!(signature.safety, Safety::Unsafe(_))
      || matches!(signature.output, ReturnType::Default)
    {
      return Err(Error::new_spanned(
        signature,
        "`#[constant]` marks a function of no arguments, not generic, async or unsafe, returning the value exported",
      ));
    }

    let name: &Ident = &signature.ident;
    let doc: String = Self::read_doc(&declared.attrs)?;

    Ok(Self::register(
      &declared,
      name,
      &name.to_string().to_uppercase(),
      &doc,
      quote!(super::#name()),
    ))
  }

  /// The item as written, then the hidden module submitting it as the constant `exported`, whose value `value` reads.
  fn register(
    declared: &impl quote::ToTokens,
    name: &Ident,
    exported: &str,
    doc: &str,
    value: TokenStream,
  ) -> TokenStream {
    // A module rather than an anonymous `const`: Rust refuses a module in an `impl` or a trait with a message naming
    // the place, where an associated constant would otherwise fail inside the generated code.
    let registration: Ident = format_ident!("__xrf_ipc_constant_{}", name);

    quote_spanned! {declared.span()=>
      #declared

      #[doc(hidden)]
      #[allow(non_snake_case)]
      mod #registration {
        ::xrf_ipc::inventory::submit! {
          ::xrf_ipc::IpcConstant {
            module_path: ::xrf_ipc::IpcConstant::enclosing_module(::core::module_path!()),
            name: #exported,
            doc: #doc,
            value: || ::xrf_ipc::IpcConstant::serialize_value(&#value),
            data_type: |types| ::xrf_ipc::IpcConstant::define_type(&#value, types),
          }
        }
      }
    }
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
}
