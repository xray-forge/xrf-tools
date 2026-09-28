use xrf_lua::{XRayLuaBinding, XRayLuaMethodCall, XRayLuaValue};

use crate::xray_shader_sampler_texture::XRayShaderSamplerTexture;

/// One sampler a pass binds, `shader:sampler("s_nmap"):texture(tex_nmap)`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct XRayShaderSampler {
  name: String,
  texture: XRayShaderSamplerTexture,
}

impl XRayShaderSampler {
  /// The chained call that binds a sampler's texture.
  const TEXTURE_METHOD: &'static str = "texture";

  /// Reads one `shader:sampler` call, its texture resolved through what the name it passes means where it passes it.
  pub fn of(call: &XRayLuaMethodCall) -> Option<Self> {
    let name: String = call.arguments().first()?.as_string()?.to_owned();
    let texture: XRayShaderSamplerTexture = match call
      .chained_call(Self::TEXTURE_METHOD)
      .and_then(|texture| texture.argument(0))
    {
      Some(XRayLuaValue::String(value))
      | Some(XRayLuaValue::Local {
        binding: XRayLuaBinding::String(value),
        ..
      }) => XRayShaderSamplerTexture::Named(value.clone()),
      Some(XRayLuaValue::Local {
        binding: XRayLuaBinding::Parameter,
        name,
      }) => XRayShaderSamplerTexture::Parameter(name.clone()),
      Some(XRayLuaValue::Name(name)) => XRayShaderSamplerTexture::Global(name.clone()),
      _ => XRayShaderSamplerTexture::Unresolved,
    };

    Some(Self { name, texture })
  }

  /// The sampler's name in the shader, `s_base` and its kin.
  pub fn name(&self) -> &str {
    &self.name
  }

  pub fn texture(&self) -> &XRayShaderSamplerTexture {
    &self.texture
  }
}
