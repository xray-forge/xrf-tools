use std::sync::Arc;

use xrf_shaders::{XRayShaderBlendFactor, XRayShaderPass, XRayShaderPassState, XRayShaderScript};
use xrf_vfs::{XrayAsset, XrayProbe};

use crate::data::xray_surface_declaration::XraySurfaceDeclaration;
use crate::data::xray_surface_descriptor::XraySurfaceDescriptor;
use crate::data::xray_surface_draw::XraySurfaceDraw;

/// The renderer shader script a surface's name resolves to, which the engine prefers over the blender library.
pub struct XraySurfaceScript;

impl XraySurfaceScript {
  /// Where the R2 renderer loads its scripts from, `getShaderPath()` under `$game_shaders$`.
  pub const SHADER_SCRIPT_DIRECTORY: &'static str = "shaders\\r2";

  /// Extension every renderer script carries, which is what `LS_Load` filters the directory by.
  pub const SHADER_SCRIPT_EXTENSION: &'static str = ".s";

  /// The delimiter a shader name separates its directory with, which the undecoration replaces.
  const NAME_DELIMITER: char = '\\';

  /// What the undecoration replaces it with, so `effects\lightplanes` is looked up as `effects_lightplanes`.
  const NAMESPACE_DELIMITER: char = '_';

  /// The logical path of the script a shader of this name would be read from.
  pub fn to_logical_path(shader_name: &str) -> String {
    format!(
      "{}\\{}{}",
      Self::SHADER_SCRIPT_DIRECTORY,
      shader_name.replace(Self::NAME_DELIMITER, &Self::NAMESPACE_DELIMITER.to_string()),
      Self::SHADER_SCRIPT_EXTENSION
    )
  }

  /// How a surface naming this shader is drawn, for a shader the roots answer with a script.
  pub fn describe(probe: &XrayProbe, shader_name: &str) -> Option<XraySurfaceDescriptor> {
    let logical_path: String = Self::to_logical_path(shader_name);
    let asset: XrayAsset = probe.find(&logical_path).ok()?.get_asset().cloned()?;
    let source: Arc<String> = probe
      .read_asset_parsed(&asset, |bytes| Ok(String::from_utf8_lossy(&bytes).into_owned()))
      .ok()?;
    let script: XRayShaderScript = XRayShaderScript::parse(&logical_path, &source).ok()?;
    let pass: &XRayShaderPass = script.pass_of(XRayShaderPass::BASE_FUNCTION)?;
    let state: &XRayShaderPassState = pass.state();

    Some(XraySurfaceDescriptor {
      shader: None,
      textures: Vec::new(),
      library: Some(asset),
      declaration: XraySurfaceDeclaration::Scripted {
        function: XRayShaderPass::BASE_FUNCTION.to_owned(),
        is_alpha_tested: state.is_alpha_tested,
        is_blended: state.is_blended,
        is_depth_written: state.is_depth_written,
        is_wallmark: state.is_wallmark,
        script: logical_path,
      },
      draw: Self::to_draw(state),
      // A script binds its own samplers by name, so nothing here is the detail texture the library's classes bind.
      detail: None,
    })
  }

  /// The pass the script declares, as the draw the renderer compiles from it.
  fn to_draw(state: &XRayShaderPassState) -> XraySurfaceDraw {
    let reference: u8 = state.alpha_reference.unwrap_or(0);

    if !state.is_blended {
      // A reference of zero discards nothing. DX10 and DX11 have no alpha test state at all - the reference is bound
      // as a shader constant and only a pixel shader calling `clip` acts on it - and a reference of zero would leave
      // even the D3D9 path with nothing to cut but fully transparent texels.
      return if state.is_alpha_tested && reference > 0 {
        XraySurfaceDraw::AlphaTested { reference }
      } else {
        XraySurfaceDraw::Opaque
      };
    }

    match (state.blend_source, state.blend_destination) {
      // Keeps what is behind and adds nothing of its own.
      (Some(XRayShaderBlendFactor::Zero), Some(XRayShaderBlendFactor::One)) => XraySurfaceDraw::Invisible,
      // Writes its own colour outright, which is the equation of a pass that is not really blending.
      (Some(XRayShaderBlendFactor::One), Some(XRayShaderBlendFactor::Zero)) => XraySurfaceDraw::Opaque,
      (Some(XRayShaderBlendFactor::SourceAlpha | XRayShaderBlendFactor::One), Some(XRayShaderBlendFactor::One)) => {
        XraySurfaceDraw::Added { reference }
      }
      (Some(XRayShaderBlendFactor::DestinationColor), Some(XRayShaderBlendFactor::Zero)) => {
        XraySurfaceDraw::Multiplied { is_doubled: false }
      }
      (Some(XRayShaderBlendFactor::DestinationColor), Some(XRayShaderBlendFactor::SourceColor)) => {
        XraySurfaceDraw::Multiplied { is_doubled: true }
      }
      // Source alpha over its inverse, and everything else a script writes: the ordinary composite, which is the one
      // answer that is wrong in the fewest ways for an equation this does not name.
      _ => XraySurfaceDraw::Blended { reference },
    }
  }
}
