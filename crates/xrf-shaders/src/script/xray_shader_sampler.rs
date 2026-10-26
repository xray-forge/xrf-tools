use xrf_lua::XRayLuaMethodCall;

use crate::script::XRayShaderSamplerTexture;

/// One texture a pass binds by name: `shader:sampler("s_nmap"):texture(tex_nmap)`, or on DX10 and later
/// `shader:dx10texture("s_nmap", tex_nmap)`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct XRayShaderSampler {
  name: String,
  texture: XRayShaderSamplerTexture,
}

impl XRayShaderSampler {
  /// The call that binds a sampler and its texture together, which every renderer's scripts may use.
  pub const SAMPLER_METHOD: &'static str = "sampler";
  /// The call DX10 and later bind a texture with, apart from the sampler state `dx10sampler` declares
  /// (`dx11ResourceManager_Scripting.cpp`: `r_dx11Texture`).
  pub const TEXTURE_METHOD: &'static str = "dx10texture";

  /// The chained call that binds a `sampler`'s texture.
  const SAMPLER_TEXTURE_METHOD: &'static str = "texture";

  /// Reads one `shader:sampler` or `shader:dx10texture` call, its texture resolved through what the name it passes
  /// means where it passes it; `None` for any other call, or one naming no sampler.
  pub fn of(call: &XRayLuaMethodCall) -> Option<Self> {
    let name: String = call.arguments().first()?.as_string()?.to_owned();
    let texture: XRayShaderSamplerTexture = match call.method() {
      Self::SAMPLER_METHOD => XRayShaderSamplerTexture::of(
        call
          .chained_call(Self::SAMPLER_TEXTURE_METHOD)
          .and_then(|texture| texture.argument(0)),
      ),
      Self::TEXTURE_METHOD => XRayShaderSamplerTexture::of(call.arguments().get(1)),
      _ => return None,
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
