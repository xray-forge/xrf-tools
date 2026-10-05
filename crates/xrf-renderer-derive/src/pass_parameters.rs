use proc_macro2::{Ident, TokenStream};
use quote::quote;
use syn::{Attribute, Data, DataStruct, DeriveInput, Error, Fields, LitInt, Meta, Type};

/// One field's binding, read from its attribute.
enum Binding {
  Uniform,
  Storage,
  Texture {
    dimension: Dimension,
    sample: Sample,
  },
  StorageTexture {
    dimension: Dimension,
    format: (TokenStream, &'static str),
    access: Access,
  },
  Sampler {
    kind: SamplerKind,
  },
}

#[derive(Clone, Copy)]
enum Dimension {
  D2,
  D2Array,
  Cube,
  CubeArray,
  D3,
}

#[derive(Clone, Copy)]
enum Sample {
  Float,
  Unfilterable,
  Uint,
  Sint,
  Depth,
}

#[derive(Clone, Copy)]
enum Access {
  Read,
  Write,
  ReadWrite,
}

#[derive(Clone, Copy)]
enum SamplerKind {
  Filtering,
  NonFiltering,
  Comparison,
}

/// The storage texture formats a parameter may name: the wgpu variant and the WGSL texel format.
const STORAGE_FORMATS: &[(&str, &str, &str)] = &[
  ("rgba8unorm", "Rgba8Unorm", "rgba8unorm"),
  ("rgba16float", "Rgba16Float", "rgba16float"),
  ("rgba32float", "Rgba32Float", "rgba32float"),
  ("rg32float", "Rg32Float", "rg32float"),
  ("r32float", "R32Float", "r32float"),
  ("r32uint", "R32Uint", "r32uint"),
  ("r32sint", "R32Sint", "r32sint"),
  ("rgba32uint", "Rgba32Uint", "rgba32uint"),
];

/// Expands `#[derive(PassParameters)]`: one bind group's layout, WGSL declarations, graph accesses and bindings, a
/// binding per field in order.
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
        "`PassParameters` can only be derived for a struct with named fields",
      ));
    }
  };
  let group: u32 = read_group(&input.attrs, name)?;
  let mut bindings: Vec<(&Ident, &Type, Binding)> = Vec::new();

  for field in fields {
    let ident: &Ident = field.ident.as_ref().expect("named fields");

    bindings.push((ident, &field.ty, read_binding(&field.attrs, ident)?));
  }

  if bindings.is_empty() {
    return Err(Error::new_spanned(
      name,
      "A `PassParameters` struct needs at least one binding",
    ));
  }

  let core: TokenStream = quote!(::xrf_renderer_core);
  let (impl_generics, type_generics, where_clause) = input.generics.split_for_impl();
  let key: String = name.to_string();
  let mut layout_entries: Vec<TokenStream> = Vec::new();
  let mut wgsl: Vec<TokenStream> = Vec::new();
  let mut declares: Vec<TokenStream> = Vec::new();
  let mut texture_accesses: Vec<TokenStream> = Vec::new();
  let mut buffer_accesses: Vec<TokenStream> = Vec::new();
  let mut resources: Vec<TokenStream> = Vec::new();
  let mut dynamic_offsets: Vec<TokenStream> = Vec::new();

  for (index, (ident, ty, binding)) in bindings.iter().enumerate() {
    let index: u32 = index as u32;
    let field_name: String = ident.to_string();
    let prefix: String = format!("@group({group}) @binding({index}) var");

    match binding {
      Binding::Uniform => {
        layout_entries.push(quote! {
          #core::PassBindingLayout::uniform::<<#ty as #core::UniformField>::Value>(#index)
        });
        wgsl.push(quote! {
          ::std::format!("{}<uniform> {}: {};\n", #prefix, #field_name,
            <<#ty as #core::UniformField>::Value as #core::ShaderType>::get_wgsl_name())
        });
        declares.push(quote!(<<#ty as #core::UniformField>::Value as #core::ShaderType>::declare(declarations);));
        resources.push(quote!(#core::UniformField::get_binding(&self.#ident)));
        dynamic_offsets.push(quote!(#core::UniformField::get_dynamic_offset(&self.#ident)));
      }
      Binding::Storage => {
        layout_entries.push(quote! {
          #core::PassBindingLayout::storage(#index, <#ty as #core::StorageField>::IS_WRITABLE)
        });
        wgsl.push(quote! {
          ::std::format!("{}<storage, {}> {}: array<{}>;\n", #prefix,
            if <#ty as #core::StorageField>::IS_WRITABLE { "read_write" } else { "read" }, #field_name,
            <<#ty as #core::StorageField>::Element as #core::ShaderType>::get_wgsl_name())
        });
        declares.push(quote!(<<#ty as #core::StorageField>::Element as #core::ShaderType>::declare(declarations);));
        buffer_accesses.push(quote! {
          (#core::StorageField::get_buffer(&self.#ident), #core::StorageField::get_access(&self.#ident))
        });
        resources.push(quote! {
          #core::PassBinding::Buffer(resources.get_buffer(#core::StorageField::get_buffer(&self.#ident)))
        });
      }
      Binding::Texture { dimension, sample } => {
        let (view, wgsl_type) = texture_view(*dimension, *sample)?;
        let sample_type: TokenStream = match sample {
          Sample::Float => quote!(::xrf_renderer_core::wgpu::TextureSampleType::Float { filterable: true }),
          Sample::Unfilterable => quote!(::xrf_renderer_core::wgpu::TextureSampleType::Float { filterable: false }),
          Sample::Uint => quote!(::xrf_renderer_core::wgpu::TextureSampleType::Uint),
          Sample::Sint => quote!(::xrf_renderer_core::wgpu::TextureSampleType::Sint),
          Sample::Depth => quote!(::xrf_renderer_core::wgpu::TextureSampleType::Depth),
        };

        layout_entries.push(quote!(#core::PassBindingLayout::texture(#index, #sample_type, #view)));
        wgsl.push(quote!(::std::format!("{} {}: {};\n", #prefix, #field_name, #wgsl_type)));
        texture_accesses.push(quote!((self.#ident, #core::GraphTextureAccess::Sampled)));
        resources.push(quote!(#core::PassBinding::TextureView(resources.get_texture(self.#ident).view)));
      }
      Binding::StorageTexture {
        dimension,
        format: (format, wgsl_format),
        access,
      } => {
        let (view, wgsl_dimension) = match dimension {
          Dimension::D2 => (quote!(::xrf_renderer_core::wgpu::TextureViewDimension::D2), "2d"),
          Dimension::D2Array => (
            quote!(::xrf_renderer_core::wgpu::TextureViewDimension::D2Array),
            "2d_array",
          ),
          Dimension::D3 => (quote!(::xrf_renderer_core::wgpu::TextureViewDimension::D3), "3d"),
          Dimension::Cube | Dimension::CubeArray => {
            return Err(Error::new_spanned(
              ident,
              "A storage texture is two- or three-dimensional",
            ));
          }
        };
        let (wgpu_access, wgsl_access, graph_access) = match access {
          Access::Read => (
            quote!(::xrf_renderer_core::wgpu::StorageTextureAccess::ReadOnly),
            "read",
            quote!(#core::GraphTextureAccess::StorageRead),
          ),
          Access::Write => (
            quote!(::xrf_renderer_core::wgpu::StorageTextureAccess::WriteOnly),
            "write",
            quote!(#core::GraphTextureAccess::StorageWrite),
          ),
          Access::ReadWrite => (
            quote!(::xrf_renderer_core::wgpu::StorageTextureAccess::ReadWrite),
            "read_write",
            quote!(#core::GraphTextureAccess::StorageReadWrite),
          ),
        };
        let wgsl_type: String = format!("texture_storage_{wgsl_dimension}<{wgsl_format}, {wgsl_access}>");
        let is_writable: bool = !matches!(access, Access::Read);

        layout_entries.push(quote! {
          #core::PassBindingLayout::storage_texture(#index, #wgpu_access, #format, #view, #is_writable)
        });
        wgsl.push(quote!(::std::format!("{} {}: {};\n", #prefix, #field_name, #wgsl_type)));
        texture_accesses.push(quote!((self.#ident, #graph_access)));
        resources.push(quote!(#core::PassBinding::TextureView(resources.get_texture(self.#ident).view)));
      }
      Binding::Sampler { kind } => {
        let (binding_type, wgsl_type) = match kind {
          SamplerKind::Filtering => (
            quote!(::xrf_renderer_core::wgpu::SamplerBindingType::Filtering),
            "sampler",
          ),
          SamplerKind::NonFiltering => (
            quote!(::xrf_renderer_core::wgpu::SamplerBindingType::NonFiltering),
            "sampler",
          ),
          SamplerKind::Comparison => (
            quote!(::xrf_renderer_core::wgpu::SamplerBindingType::Comparison),
            "sampler_comparison",
          ),
        };

        layout_entries.push(quote!(#core::PassBindingLayout::sampler(#index, #binding_type)));
        wgsl.push(quote!(::std::format!("{} {}: {};\n", #prefix, #field_name, #wgsl_type)));
        resources.push(quote!(#core::PassBinding::Sampler(self.#ident)));
      }
    }
  }

  Ok(quote! {
    impl #impl_generics #core::PassParameters for #name #type_generics #where_clause {
      const GROUP: u32 = #group;
      const LAYOUT_KEY: &'static str = ::core::concat!(::core::module_path!(), "::", #key);

      fn get_layout_entries() -> ::std::vec::Vec<::xrf_renderer_core::wgpu::BindGroupLayoutEntry> {
        ::std::vec![#(#layout_entries),*]
      }

      fn get_wgsl_bindings() -> ::std::string::String {
        let mut wgsl: ::std::string::String = ::std::string::String::new();

        #(wgsl.push_str(&#wgsl);)*
        wgsl
      }

      fn declare(declarations: &mut #core::ShaderDeclarations) {
        #(#declares)*
      }

      fn list_texture_accesses(&self) -> ::std::vec::Vec<(#core::GraphTexture, #core::GraphTextureAccess)> {
        ::std::vec![#(#texture_accesses),*]
      }

      fn list_buffer_accesses(&self) -> ::std::vec::Vec<(#core::GraphBuffer, #core::GraphBufferAccess)> {
        ::std::vec![#(#buffer_accesses),*]
      }

      fn list_bindings<'r>(&'r self, resources: &dyn #core::PassResources<'r>) -> ::std::vec::Vec<#core::PassBinding<'r>> {
        let _ = resources;

        ::std::vec![#(#resources),*]
      }

      fn list_dynamic_offsets(&self) -> ::std::vec::Vec<u32> {
        ::std::vec![#(#dynamic_offsets),*]
      }
    }
  })
}

fn read_group(attrs: &[Attribute], name: &Ident) -> syn::Result<u32> {
  let mut group: Option<u32> = None;

  for attr in attrs.iter().filter(|attr| attr.path().is_ident("parameters")) {
    attr.parse_nested_meta(|meta| {
      if meta.path.is_ident("group") {
        group = Some(meta.value()?.parse::<LitInt>()?.base10_parse()?);
        Ok(())
      } else {
        Err(meta.error("expected `group = <index>`"))
      }
    })?;
  }

  group.ok_or_else(|| Error::new_spanned(name, "`PassParameters` needs `#[parameters(group = <index>)]`"))
}

fn read_binding(attrs: &[Attribute], ident: &Ident) -> syn::Result<Binding> {
  let mut binding: Option<Binding> = None;

  for attr in attrs {
    let path = attr.path();
    let found: Option<Binding> = if path.is_ident("uniform") {
      Some(Binding::Uniform)
    } else if path.is_ident("storage") {
      Some(Binding::Storage)
    } else if path.is_ident("texture") {
      let words: Vec<String> = read_words(attr)?;

      Some(Binding::Texture {
        dimension: read_dimension(attr, &words)?,
        sample: read_sample(attr, &words)?,
      })
    } else if path.is_ident("storage_texture") {
      let words: Vec<String> = read_words(attr)?;
      let format = words
        .iter()
        .find_map(|word| STORAGE_FORMATS.iter().find(|(spelling, _, _)| spelling == word))
        .ok_or_else(|| Error::new_spanned(attr, "a storage texture names its format, as `rgba16float`"))?;
      let variant: Ident = Ident::new(format.1, proc_macro2::Span::call_site());

      Some(Binding::StorageTexture {
        dimension: read_dimension(attr, &words)?,
        format: (quote!(::xrf_renderer_core::wgpu::TextureFormat::#variant), format.2),
        access: match () {
          _ if words.iter().any(|word| word == "read_write") => Access::ReadWrite,
          _ if words.iter().any(|word| word == "write") => Access::Write,
          _ if words.iter().any(|word| word == "read") => Access::Read,
          _ => {
            return Err(Error::new_spanned(
              attr,
              "a storage texture is `read`, `write` or `read_write`",
            ));
          }
        },
      })
    } else if path.is_ident("sampler") {
      let words: Vec<String> = read_words(attr)?;

      Some(Binding::Sampler {
        kind: match words.first().map(String::as_str) {
          Some("filtering") => SamplerKind::Filtering,
          Some("non_filtering") => SamplerKind::NonFiltering,
          Some("comparison") => SamplerKind::Comparison,
          _ => {
            return Err(Error::new_spanned(
              attr,
              "a sampler is `filtering`, `non_filtering` or `comparison`",
            ));
          }
        },
      })
    } else {
      None
    };

    if let Some(found) = found {
      if binding.is_some() {
        return Err(Error::new_spanned(attr, "a field is one binding"));
      }

      binding = Some(found);
    }
  }

  binding.ok_or_else(|| {
    Error::new_spanned(
      ident,
      "a parameter is `#[uniform]`, `#[storage]`, `#[texture(..)]`, `#[storage_texture(..)]` or `#[sampler(..)]`",
    )
  })
}

/// The words inside an attribute's parentheses, as `d2, float`.
fn read_words(attr: &Attribute) -> syn::Result<Vec<String>> {
  let mut words: Vec<String> = Vec::new();

  if let Meta::List(_) = &attr.meta {
    attr.parse_nested_meta(|meta| {
      words.push(
        meta
          .path
          .get_ident()
          .map(Ident::to_string)
          .ok_or_else(|| meta.error("expected a word"))?,
      );
      Ok(())
    })?;
  }

  Ok(words)
}

fn read_dimension(attr: &Attribute, words: &[String]) -> syn::Result<Dimension> {
  words
    .iter()
    .find_map(|word| match word.as_str() {
      "d2" => Some(Dimension::D2),
      "d2_array" => Some(Dimension::D2Array),
      "cube" => Some(Dimension::Cube),
      "cube_array" => Some(Dimension::CubeArray),
      "d3" => Some(Dimension::D3),
      _ => None,
    })
    .ok_or_else(|| {
      Error::new_spanned(
        attr,
        "a texture names its dimension: `d2`, `d2_array`, `cube`, `cube_array` or `d3`",
      )
    })
}

fn read_sample(attr: &Attribute, words: &[String]) -> syn::Result<Sample> {
  words
    .iter()
    .find_map(|word| match word.as_str() {
      "float" => Some(Sample::Float),
      "unfilterable" => Some(Sample::Unfilterable),
      "uint" => Some(Sample::Uint),
      "sint" => Some(Sample::Sint),
      "depth" => Some(Sample::Depth),
      _ => None,
    })
    .ok_or_else(|| {
      Error::new_spanned(
        attr,
        "a texture names what it samples: `float`, `unfilterable`, `uint`, `sint` or `depth`",
      )
    })
}

/// The view dimension and WGSL type of a sampled texture.
fn texture_view(dimension: Dimension, sample: Sample) -> syn::Result<(TokenStream, String)> {
  let (view, name): (TokenStream, &str) = match dimension {
    Dimension::D2 => (quote!(::xrf_renderer_core::wgpu::TextureViewDimension::D2), "2d"),
    Dimension::D2Array => (
      quote!(::xrf_renderer_core::wgpu::TextureViewDimension::D2Array),
      "2d_array",
    ),
    Dimension::Cube => (quote!(::xrf_renderer_core::wgpu::TextureViewDimension::Cube), "cube"),
    Dimension::CubeArray => (
      quote!(::xrf_renderer_core::wgpu::TextureViewDimension::CubeArray),
      "cube_array",
    ),
    Dimension::D3 => (quote!(::xrf_renderer_core::wgpu::TextureViewDimension::D3), "3d"),
  };
  let wgsl: String = match sample {
    Sample::Depth if matches!(dimension, Dimension::D3) => {
      return Err(Error::new(
        proc_macro2::Span::call_site(),
        "A depth texture is not three-dimensional",
      ));
    }
    Sample::Depth => format!("texture_depth_{name}"),
    Sample::Float | Sample::Unfilterable => format!("texture_{name}<f32>"),
    Sample::Uint => format!("texture_{name}<u32>"),
    Sample::Sint => format!("texture_{name}<i32>"),
  };

  Ok((view, wgsl))
}
