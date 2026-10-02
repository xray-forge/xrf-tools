use std::sync::Arc;

use xrf_shaders::{XRayShaderBlendFactor, XRayShaderPass, XRayShaderPassState, XRayShaderSampler, XRayShaderScript};
use xrf_vfs::{XrayAsset, XrayProbe};

use crate::data::xray_material_descriptor::XrayMaterialDescriptor;
use crate::data::xray_surface_declaration::XraySurfaceDeclaration;
use crate::data::xray_surface_descriptor::XraySurfaceDescriptor;
use crate::data::xray_surface_draw::XraySurfaceDraw;
use crate::data::xray_surface_sampler::XraySurfaceSampler;

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

  /// The function whose pass the renderer compiles into its distortion target (`L_special`, `mapDistort`).
  pub const DISTORTION_FUNCTION: &'static str = "l_special";

  /// The engine's water program (`shaders/r2/water.vs`), and what every variant of it is named behind: vanilla's
  /// `water_soft`, Anomaly's `water_regular`, `water_studen`, `water_ryaska` and `water_underground`.
  const WATER_PROGRAM: &'static str = "water";
  const WATER_VARIANT_PREFIX: &'static str = "water_";

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
    let samplers: Vec<XraySurfaceSampler> = [XRayShaderPass::BASE_FUNCTION, Self::DISTORTION_FUNCTION]
      .into_iter()
      .filter_map(|function| script.pass_of(function))
      .flat_map(Self::to_samplers)
      .collect();

    Some(XraySurfaceDescriptor {
      shader: None,
      textures: Vec::new(),
      library: Some(asset),
      declaration: XraySurfaceDeclaration::Scripted {
        function: XRayShaderPass::BASE_FUNCTION.to_owned(),
        program: pass.vertex_shader().to_owned(),
        is_alpha_tested: state.is_alpha_tested,
        is_blended: state.is_blended,
        is_depth_written: state.is_depth_written,
        is_wallmark: state.is_wallmark,
        script: logical_path,
      },
      draw: Self::to_water_draw(pass).unwrap_or_else(|| Self::to_draw(state)),
      // A script binds its own samplers by name, so nothing here is the detail texture the library's classes bind.
      detail: None,
      samplers,
      bump: None,
      material: XrayMaterialDescriptor::DEFAULT_MATERIAL,
      environment: None,
    })
  }

  /// The texture files one pass binds, each by its sampler.
  fn to_samplers(pass: &XRayShaderPass) -> Vec<XraySurfaceSampler> {
    let element: &str = pass.function().unwrap_or_default();

    pass
      .samplers()
      .iter()
      .filter_map(|sampler: &XRayShaderSampler| {
        Some(XraySurfaceSampler {
          element: element.to_owned(),
          name: sampler.name().to_owned(),
          texture: sampler.texture().file()?.to_owned(),
        })
      })
      .collect()
  }

  /// Water, for a pass drawn by a water program, which no blend state tells apart from a plain surface. Soft where
  /// it blends: `water_soft` is vanilla's one blended program and the only one reading the depth behind it, while
  /// vanilla's plain `water` draws `blend(false)`.
  fn to_water_draw(pass: &XRayShaderPass) -> Option<XraySurfaceDraw> {
    let program: &str = pass.vertex_shader();

    (program == Self::WATER_PROGRAM || program.starts_with(Self::WATER_VARIANT_PREFIX)).then_some(
      XraySurfaceDraw::Water {
        is_soft: pass.state().is_blended,
      },
    )
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
      (Some(XRayShaderBlendFactor::SourceAlpha), Some(XRayShaderBlendFactor::One)) => XraySurfaceDraw::Added {
        is_weighted: true,
        reference,
      },
      (Some(XRayShaderBlendFactor::One), Some(XRayShaderBlendFactor::One)) => XraySurfaceDraw::Added {
        is_weighted: false,
        reference,
      },
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
