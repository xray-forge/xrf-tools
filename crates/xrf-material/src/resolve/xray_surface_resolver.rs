use std::sync::Arc;

use xrf_chunk::ChunkReader;
use xrf_error::XrfResult;
use xrf_shaders::{ShaderBlender, ShaderLibraryFile};
use xrf_vfs::{XrayAsset, XrayAssetType, XrayProbe};

use crate::data::xray_detail_usage::XrayDetailUsage;
use crate::data::xray_material_bump::XrayMaterialBump;
use crate::data::xray_material_descriptor::XrayMaterialDescriptor;
use crate::data::xray_material_detail::XrayMaterialDetail;
use crate::data::xray_surface_declaration::XraySurfaceDeclaration;
use crate::data::xray_surface_descriptor::XraySurfaceDescriptor;
use crate::data::xray_surface_detail::XraySurfaceDetail;
use crate::resolve::xray_material_resolver::XrayMaterialResolver;
use crate::resolve::xray_surface_alpha::XraySurfaceAlpha;
use crate::resolve::xray_surface_bump_rule::XraySurfaceBumpRule;
use crate::resolve::xray_surface_detail_rule::XraySurfaceDetailRule;
use crate::resolve::xray_surface_rule::XraySurfaceRule;
use crate::resolve::xray_surface_script::XraySurfaceScript;
use crate::resolve::xray_texture_scope::XrayTextureScope;

/// Answers how a surface is drawn from the shader name it declares and the textures it dresses with.
pub struct XraySurfaceResolver<'probe, 'vfs> {
  probe: &'probe XrayProbe<'vfs>,
  scope: XrayTextureScope,
  source: XraySurfaceSource,
}

impl<'probe, 'vfs> XraySurfaceResolver<'probe, 'vfs> {
  /// Where the engine loads the blender library from: the game data root, beside `gamemtl.xr`
  /// (`Layers/xrRender/ResourceManager_Loader.cpp`).
  pub const SHADER_LIBRARY_LOGICAL_PATH: &'static str = XrayAssetType::SHADER_LIBRARY_PATH;

  /// Locates and reads the shader library once, recording why it could not rather than failing, for surfaces whose
  /// textures are read within `scope`.
  pub fn open(probe: &'probe XrayProbe<'vfs>, scope: XrayTextureScope) -> Self {
    let Some(asset) = probe
      .find(Self::SHADER_LIBRARY_LOGICAL_PATH)
      .ok()
      .and_then(|resolution| resolution.get_asset().cloned())
    else {
      return Self {
        probe,
        scope,
        source: XraySurfaceSource::Absent,
      };
    };

    Self {
      probe,
      scope,
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
    if let Some(scripted) = XraySurfaceScript::describe(self.probe, shader_name, textures) {
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
    // The base texture's descriptor, which says the detail, the bump pair and the lighting model all at once.
    let base: Option<XrayMaterialDescriptor> = blender
      .base_texture(textures)
      .map(|base| XrayMaterialResolver::describe_texture(self.probe, &self.scope, base));
    let bump: Option<XrayMaterialBump> = base
      .as_ref()
      .filter(|_| XraySurfaceBumpRule::is_bumped(blender.class))
      .and_then(|descriptor| descriptor.bump.clone());

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
      detail: base
        .as_ref()
        .and_then(|descriptor| self.describe_detail(blender, descriptor, bump.is_some())),
      samplers: Vec::new(),
      bump,
      material: base.map_or(XrayMaterialDescriptor::DEFAULT_MATERIAL, |descriptor| {
        descriptor.material
      }),
      environment: rule.environment(blender),
      is_texture_clamped: rule.is_texture_clamped(blender),
      is_distorting: false,
      is_object_lod: rule.is_object_lod(blender),
    }
  }

  /// The detail texture the blender's class binds, laid out at the tiling its base texture's descriptor sets, with the
  /// bump pair its own descriptor names where its usage bumps a surface bumped itself.
  fn describe_detail(
    &self,
    blender: &ShaderBlender,
    descriptor: &XrayMaterialDescriptor,
    is_bumped: bool,
  ) -> Option<XraySurfaceDetail> {
    let rule: XraySurfaceDetailRule = XraySurfaceDetailRule::of(blender.class)?;
    let associated: &XrayMaterialDetail = descriptor
      .detail
      .as_ref()
      .filter(|_| descriptor.is_detail_associated())?;
    let reference: String = rule.reference(blender, Some(associated.name.as_str()))?.to_owned();
    // `r2_detail_bump` is on by default, which leaves a bumping usage bumping (`Blender_Recorder.cpp`).
    let is_detail_bumped: bool = is_bumped
      && matches!(
        associated.usage,
        Some(XrayDetailUsage::Bump | XrayDetailUsage::DiffuseAndBump)
      );
    let bump: Option<XrayMaterialBump> = is_detail_bumped
      .then(|| XrayMaterialResolver::describe_texture(self.probe, &self.scope, &reference).bump)
      .flatten();

    Some(XraySurfaceDetail {
      bump,
      reference,
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
