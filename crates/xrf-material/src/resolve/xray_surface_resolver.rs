use std::sync::Arc;

use xrf_chunk::ChunkReader;
use xrf_db::{ShaderBlender, ShaderLibraryFile};
use xrf_error::XrfResult;
use xrf_vfs::{XrayAsset, XrayProbe};

use crate::data::xray_surface_declaration::XraySurfaceDeclaration;
use crate::data::xray_surface_descriptor::XraySurfaceDescriptor;
use crate::resolve::xray_surface_alpha::XraySurfaceAlpha;
use crate::resolve::xray_surface_rule::XraySurfaceRule;

/// Answers how a surface is drawn from the shader name it declares.
///
/// Opened once and asked many times, unlike [`crate::XrayMaterialResolver`]: a texture's answer is a file of its own,
/// while every surface of every model in a tree is answered by the one `shaders.xr`. Reading it per submesh would
/// re-read a 190KB+ file forty times for one model.
///
/// Reads through a borrowed probe and never mounts: which roots exist, in what order, is the caller's policy.
///
/// # What it models
///
/// The deferred renderer, R2 and above, which is what the game runs. The engine's rules live in
/// [`XraySurfaceRule`]; this locates the library, reads it, and looks a name up in it. The two halves of an answer are
/// kept apart on purpose: [`XraySurfaceDeclaration`] is what the author wrote and
/// [`crate::XraySurfaceDraw`] is what that comes to, so a surface can be shown as authored and as drawn without one
/// being inferred back out of the other.
pub struct XraySurfaceResolver {
  source: XraySurfaceSource,
}

impl XraySurfaceResolver {
  /// Where the engine loads the blender library from: the game data root, beside `gamemtl.xr`
  /// (`Layers/xrRender/ResourceManager_Loader.cpp`).
  pub const SHADER_LIBRARY_LOGICAL_PATH: &'static str = "shaders.xr";

  /// Locates and reads the shader library once, recording why it could not rather than failing.
  ///
  /// A tree with no `shaders.xr` is an ordinary thing to open - a loose folder of meshes, an archive of models - and
  /// it costs the answer for every surface, not the model.
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
  ///
  /// The name is taken as the mesh spells it, which is how the engine looks it up: `CResourceManager::_GetBlender`
  /// keys the library by the string in the OGF texture chunk.
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
