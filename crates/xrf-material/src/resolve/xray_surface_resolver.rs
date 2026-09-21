use std::sync::Arc;

use xrf_chunk::ChunkReader;
use xrf_error::XrfResult;
use xrf_shaders::{ShaderBlender, ShaderLibraryFile};
use xrf_vfs::{XrayAsset, XrayAssetType, XrayProbe};

use crate::data::xray_material_descriptor::XrayMaterialDescriptor;
use crate::data::xray_material_detail::XrayMaterialDetail;
use crate::data::xray_surface_declaration::XraySurfaceDeclaration;
use crate::data::xray_surface_descriptor::XraySurfaceDescriptor;
use crate::data::xray_surface_detail::XraySurfaceDetail;
use crate::resolve::xray_material_resolver::XrayMaterialResolver;
use crate::resolve::xray_surface_alpha::XraySurfaceAlpha;
use crate::resolve::xray_surface_detail_rule::XraySurfaceDetailRule;
use crate::resolve::xray_surface_rule::XraySurfaceRule;
use crate::resolve::xray_surface_script::XraySurfaceScript;

/// Answers how a surface is drawn from the shader name it declares and the textures it dresses with.
pub struct XraySurfaceResolver<'probe, 'vfs> {
  probe: &'probe XrayProbe<'vfs>,
  source: XraySurfaceSource,
}

impl<'probe, 'vfs> XraySurfaceResolver<'probe, 'vfs> {
  /// Where the engine loads the blender library from: the game data root, beside `gamemtl.xr`
  /// (`Layers/xrRender/ResourceManager_Loader.cpp`).
  pub const SHADER_LIBRARY_LOGICAL_PATH: &'static str = XrayAssetType::SHADER_LIBRARY_PATH;

  /// Locates and reads the shader library once, recording why it could not rather than failing.
  pub fn open(probe: &'probe XrayProbe<'vfs>) -> Self {
    let Some(asset) = probe
      .find(Self::SHADER_LIBRARY_LOGICAL_PATH)
      .ok()
      .and_then(|resolution| resolution.get_asset().cloned())
    else {
      return Self {
        probe,
        source: XraySurfaceSource::Absent,
      };
    };

    Self {
      probe,
      source: match Self::read(probe, &asset) {
        Ok(library) => XraySurfaceSource::Read { asset, library },
        Err(error) => XraySurfaceSource::Unreadable {
          asset,
          reason: error.to_string(),
        },
      },
    }
  }

  /// How the renderer draws a surface naming this shader and dressed with these textures.
  pub fn describe(&self, shader_name: &str, textures: &[String]) -> XraySurfaceDescriptor {
    if shader_name.is_empty() {
      return XraySurfaceDescriptor::opaque(None, XraySurfaceDeclaration::Undeclared);
    }

    self
      .describe_named(shader_name, textures)
      .resolved_from(shader_name, textures)
  }

  /// The same answer, before it is told which name it answered for.
  fn describe_named(&self, shader_name: &str, textures: &[String]) -> XraySurfaceDescriptor {
    // Before the library, because that is the order the engine asks in: a shader with a renderer script **is** that
    // script, and what `shaders.xr` calls its class never reaches the screen. Reading the class alone drew X-Ray's
    // additive glows and its wall marks as opaque black.
    if let Some(scripted) = XraySurfaceScript::describe(self.probe, shader_name) {
      return scripted;
    }

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

    self.describe_blender(asset, blender, textures)
  }

  /// What one blender declares, and what the deferred renderer compiles from it.
  fn describe_blender(&self, asset: &XrayAsset, blender: &ShaderBlender, textures: &[String]) -> XraySurfaceDescriptor {
    let class: String = blender.class.tag();
    let Some(rule) = XraySurfaceRule::of(blender.class) else {
      return XraySurfaceDescriptor::opaque(Some(asset.clone()), XraySurfaceDeclaration::Unmodelled { class });
    };

    let alpha: XraySurfaceAlpha = rule.read(blender);

    XraySurfaceDescriptor {
      shader: None,
      textures: Vec::new(),
      library: Some(asset.clone()),
      declaration: XraySurfaceDeclaration::Described {
        class,
        is_alpha_used: alpha.is_used,
        alpha_reference: alpha.reference,
        is_strict_sorting: alpha.is_strict_sorting,
      },
      draw: rule.draw(blender, alpha),
      detail: self.describe_detail(blender, textures),
    }
  }

  /// The detail texture the blender's class binds, laid out at the tiling its base texture's descriptor sets.
  fn describe_detail(&self, blender: &ShaderBlender, textures: &[String]) -> Option<XraySurfaceDetail> {
    let rule: XraySurfaceDetailRule = XraySurfaceDetailRule::of(blender.class)?;
    let base: &str = blender.base_texture(textures)?;
    let descriptor: XrayMaterialDescriptor = XrayMaterialResolver::describe_texture(self.probe, base);
    let associated: &XrayMaterialDetail = descriptor
      .detail
      .as_ref()
      .filter(|_| descriptor.is_detail_associated())?;

    Some(XraySurfaceDetail {
      reference: rule.reference(blender, Some(associated.name.as_str()))?.to_owned(),
      scale: associated.scale,
    })
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
