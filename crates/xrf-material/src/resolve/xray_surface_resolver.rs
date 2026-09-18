use std::sync::Arc;

use xrf_chunk::ChunkReader;
use xrf_error::XrfResult;
use xrf_shaders::{ShaderBlender, ShaderLibraryFile};
use xrf_vfs::{XrayAsset, XrayAssetType, XrayProbe};

use crate::data::xray_surface_declaration::XraySurfaceDeclaration;
use crate::data::xray_surface_descriptor::XraySurfaceDescriptor;
use crate::resolve::xray_surface_alpha::XraySurfaceAlpha;
use crate::resolve::xray_surface_rule::XraySurfaceRule;

/// Answers how a surface is drawn from the shader name it declares.
pub struct XraySurfaceResolver {
  source: XraySurfaceSource,
}

impl XraySurfaceResolver {
  /// Where the engine loads the blender library from: the game data root, beside `gamemtl.xr`
  /// (`Layers/xrRender/ResourceManager_Loader.cpp`).
  pub const SHADER_LIBRARY_LOGICAL_PATH: &'static str = XrayAssetType::SHADER_LIBRARY_PATH;

  /// Locates and reads the shader library once, recording why it could not rather than failing.
  pub fn open(probe: &XrayProbe) -> Self {
    let Some(asset) = probe
      .find(Self::SHADER_LIBRARY_LOGICAL_PATH)
      .ok()
      .and_then(|resolution| resolution.get_asset().cloned())
    else {
      return Self {
        source: XraySurfaceSource::Absent,
      };
    };

    Self {
      source: match Self::read(probe, &asset) {
        Ok(library) => XraySurfaceSource::Read { asset, library },
        Err(error) => XraySurfaceSource::Unreadable {
          asset,
          reason: error.to_string(),
        },
      },
    }
  }

  /// How the renderer draws a surface naming this shader.
  pub fn describe(&self, shader_name: &str) -> XraySurfaceDescriptor {
    let (asset, library): (&XrayAsset, &ShaderLibraryFile) = match &self.source {
      XraySurfaceSource::Absent => return XraySurfaceDescriptor::opaque(None, XraySurfaceDeclaration::NoLibrary),
      XraySurfaceSource::Unreadable { asset, reason } => {
        let declaration: XraySurfaceDeclaration = XraySurfaceDeclaration::Unreadable { reason: reason.clone() };

        return XraySurfaceDescriptor::opaque(Some(asset.clone()), declaration);
      }
      XraySurfaceSource::Read { asset, library } => (asset, library),
    };

    let Some(blender) = library.find_blender(shader_name) else {
      return XraySurfaceDescriptor::opaque(Some(asset.clone()), XraySurfaceDeclaration::Undefined);
    };

    Self::describe_blender(asset, blender)
  }

  /// What one blender declares, and what the deferred renderer compiles from it.
  fn describe_blender(asset: &XrayAsset, blender: &ShaderBlender) -> XraySurfaceDescriptor {
    let class: String = blender.class.tag();
    let Some(rule) = XraySurfaceRule::of(blender.class) else {
      return XraySurfaceDescriptor::opaque(Some(asset.clone()), XraySurfaceDeclaration::Unmodelled { class });
    };

    let alpha: XraySurfaceAlpha = rule.read(blender);

    XraySurfaceDescriptor {
      library: Some(asset.clone()),
      declaration: XraySurfaceDeclaration::Described {
        class,
        is_alpha_used: alpha.is_used,
        alpha_reference: alpha.reference,
        is_strict_sorting: alpha.is_strict_sorting,
      },
      draw: rule.draw(alpha),
    }
  }

  fn read(probe: &XrayProbe, asset: &XrayAsset) -> XrfResult<Arc<ShaderLibraryFile>> {
    probe.read_asset_parsed(asset, |bytes| {
      ShaderLibraryFile::read_from_chunk(&mut ChunkReader::from_vec(bytes)?)
    })
  }
}

/// The shader library as one open found it: absent, unreadable, or read.
enum XraySurfaceSource {
  Absent,
  Unreadable {
    asset: XrayAsset,
    reason: String,
  },
  Read {
    asset: XrayAsset,
    library: Arc<ShaderLibraryFile>,
  },
}
