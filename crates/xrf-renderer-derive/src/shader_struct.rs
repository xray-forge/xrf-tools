use proc_macro2::{Ident, Span, TokenStream};
use quote::{format_ident, quote};
use syn::{Attribute, Data, DataStruct, DeriveInput, Error, Fields, LitStr, Type};

/// Expands `#[derive(ShaderStruct)]`: WGSL's layout of the struct computed in constants from each member's
/// `ShaderType`, every member's Rust offset and the struct's size asserted against it, and the WGSL declaration, named as
/// the struct is unless `#[shader(name = "...")]` names it.
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
        "`ShaderStruct` can only be derived for a struct with named fields",
      ));
    }
  };

  if !input.generics.params.is_empty() {
    return Err(Error::new_spanned(
      &input.generics,
      "`ShaderStruct` cannot be derived for a generic struct",
    ));
  }

  // Fields named with a leading underscore are padding: in the Rust struct, never in WGSL, which pads implicitly.
  let members: Vec<(&Ident, &Type)> = fields
    .iter()
    .filter_map(|field| field.ident.as_ref().map(|ident| (ident, &field.ty)))
    .filter(|(ident, _)| !ident.to_string().starts_with('_'))
    .collect();

  if members.is_empty() {
    return Err(Error::new_spanned(name, "A `ShaderStruct` needs at least one member"));
  }

  let core: TokenStream = quote!(::xrf_renderer_core);
  let offsets: Vec<Ident> = (0..members.len())
    .map(|index| format_ident!("__OFFSET_{}", index, span = Span::call_site()))
    .collect();
  let offset_definitions = members.iter().enumerate().map(|(index, (_, ty))| {
    let offset: &Ident = &offsets[index];

    match index {
      0 => quote!(const #offset: u64 = 0;),
      _ => {
        let (previous, previous_ty): (&Ident, &Type) = (&offsets[index - 1], members[index - 1].1);

        quote! {
          const #offset: u64 = #core::round_up(
            <#ty as #core::ShaderType>::ALIGN,
            #previous + <#previous_ty as #core::ShaderType>::SIZE,
          );
        }
      }
    }
  });
  let types: Vec<&Type> = members.iter().map(|(_, ty)| *ty).collect();
  let last_offset: &Ident = offsets.last().expect("a member at least");
  let last_ty: &Type = types.last().expect("a member at least");
  let member_names: Vec<String> = members.iter().map(|(ident, _)| ident.to_string()).collect();
  let member_idents: Vec<&Ident> = members.iter().map(|(ident, _)| *ident).collect();
  let struct_name: String = name.to_string();
  let wgsl_name: String = read_wgsl_name(&input.attrs)?.unwrap_or_else(|| struct_name.clone());
  let offset_messages: Vec<String> = member_names
    .iter()
    .map(|member| format!("`{struct_name}.{member}` is not where WGSL places it: pad before it with `_`-named fields"))
    .collect();
  let size_message: String =
    format!("`{struct_name}` is not the size WGSL makes it: pad its end with `_`-named fields");

  Ok(quote! {
    const _: () = {
      #(#offset_definitions)*

      const __ALIGN: u64 = #core::max_of(&[#(<#types as #core::ShaderType>::ALIGN),*]);
      const __SIZE: u64 = #core::round_up(__ALIGN, #last_offset + <#last_ty as #core::ShaderType>::SIZE);

      #(assert!(::core::mem::offset_of!(#name, #member_idents) as u64 == #offsets, #offset_messages);)*
      assert!(::core::mem::size_of::<#name>() as u64 == __SIZE, #size_message);

      impl #core::ShaderType for #name {
        const ALIGN: u64 = __ALIGN;
        const SIZE: u64 = __SIZE;
        const IS_STRUCT: bool = true;

        fn get_wgsl_name() -> ::std::string::String {
          ::std::string::String::from(#wgsl_name)
        }

        fn declare(declarations: &mut #core::ShaderDeclarations) {
          #(<#types as #core::ShaderType>::declare(declarations);)*
          declarations.declare_struct::<Self>();
        }
      }

      impl #core::ShaderStruct for #name {
        const MEMBERS: &'static [#core::ShaderMember] = &[
          #(#core::ShaderMember {
            name: #member_names,
            offset: #offsets,
            size: <#types as #core::ShaderType>::SIZE,
            get_wgsl_name: <#types as #core::ShaderType>::get_wgsl_name,
          }),*
        ];
      }
    };
  })
}

/// The WGSL name `#[shader(name = "...")]` gives the struct, if it gives one.
fn read_wgsl_name(attrs: &[Attribute]) -> syn::Result<Option<String>> {
  let mut name: Option<String> = None;

  for attr in attrs.iter().filter(|attr| attr.path().is_ident("shader")) {
    attr.parse_nested_meta(|meta| {
      if meta.path.is_ident("name") {
        name = Some(meta.value()?.parse::<LitStr>()?.value());
        Ok(())
      } else {
        Err(meta.error("expected `name = \"<WGSL name>\"`"))
      }
    })?;
  }

  Ok(name)
}
