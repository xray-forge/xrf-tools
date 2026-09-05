use serde_json::Value;
use xrf_chunk::XRayByteOrder;
use xrf_error::XrfResult;

use crate::thm::chunks::thm_bump_chunk::ThmBumpChunk;
use crate::thm::chunks::thm_detail_chunk::ThmDetailChunk;
use crate::thm::chunks::thm_material_chunk::ThmMaterialChunk;
use crate::thm::chunks::thm_texture_param_chunk::ThmTextureParamChunk;
use crate::thm::tests::fixtures;
use crate::thm::thm_bump_mode::ThmBumpMode;
use crate::thm::thm_detail_usage::ThmDetailUsage;
use crate::thm::thm_file::ThmFile;
use crate::thm::thm_format::ThmFormat;
use crate::thm::thm_material::ThmMaterial;
use crate::thm::thm_mip_filter::ThmMipFilter;
use crate::thm::thm_texture_flag::ThmTextureFlag;
use crate::thm::thm_texture_flags::ThmTextureFlags;
use crate::thm::thm_texture_type::ThmTextureType;

#[test]
fn reads_every_field_the_format_defines() -> XrfResult {
  let file: ThmFile = ThmFile::read_from_bytes::<XRayByteOrder>(fixtures::descriptor())?;

  assert_eq!(file.version, Some(ThmFile::VERSION));
  assert_eq!(file.thumbnail, None);
  assert_eq!(file.thumbnail_type, Some(ThmFile::THUMBNAIL_TYPE_TEXTURE));
  assert_eq!(
    file.texture_param,
    Some(ThmTextureParamChunk {
      format: ThmFormat::Dxt5,
      flags: ThmTextureFlags::none()
        .with(ThmTextureFlag::GenerateMipMaps, true)
        .with(ThmTextureFlag::DitherColor, true)
        .with(ThmTextureFlag::HasAlpha, true),
      border_color: 0,
      fade_color: 0,
      fade_amount: 0,
      mip_filter: ThmMipFilter::Box,
      width: 512,
      height: 512,
    })
  );
  assert_eq!(file.texture_type, Some(ThmTextureType::Image));
  assert_eq!(
    file.detail,
    Some(ThmDetailChunk {
      name: String::new(),
      scale: 1.0
    })
  );
  assert_eq!(
    file.material,
    Some(ThmMaterialChunk {
      material: ThmMaterial::BlinPhong,
      weight: 0.0
    })
  );
  assert_eq!(
    file.bump,
    Some(ThmBumpChunk {
      virtual_height: ThmBumpChunk::DEFAULT_VIRTUAL_HEIGHT,
      mode: ThmBumpMode::Use,
      name: String::from(fixtures::BUMP_NAME),
    })
  );
  assert_eq!(file.ext_normal_map_name, Some(String::new()));
  assert_eq!(file.fade_delay, Some(0));
  assert!(file.extra.is_empty());

  Ok(())
}

#[test]
fn starts_a_new_descriptor_at_the_sdk_defaults() {
  // `STextureParams::STextureParams` zeroes the struct and then sets these, so a descriptor this crate creates for a
  // texture that has none is the one the SDK would have created.
  let file: ThmFile = ThmFile::new_texture();
  let param: ThmTextureParamChunk = file
    .texture_param
    .expect("Expect a new descriptor to carry texture params");

  assert_eq!(param.format, ThmFormat::Dxt1);
  assert_eq!(
    param.flags,
    ThmTextureFlags::none()
      .with(ThmTextureFlag::GenerateMipMaps, true)
      .with(ThmTextureFlag::DitherColor, true)
  );
  assert_eq!(param.mip_filter, ThmMipFilter::Box);
  assert_eq!((param.width, param.height), (0, 0));

  assert_eq!(file.version, Some(ThmFile::VERSION));
  assert_eq!(file.thumbnail_type, Some(ThmFile::THUMBNAIL_TYPE_TEXTURE));
  assert_eq!(file.texture_type, Some(ThmTextureType::Image));
  assert_eq!(file.detail.map(|detail| detail.scale), Some(1.0));
  assert_eq!(
    file.material.map(|material| material.material),
    Some(ThmMaterial::BlinPhong)
  );
  assert_eq!(
    file.bump.map(|bump| (bump.mode, bump.virtual_height)),
    Some((ThmBumpMode::None, 0.05))
  );
}

#[test]
fn keeps_a_value_the_sdk_has_no_name_for() {
  // 59 descriptors of the workspace corpus declare material 6, which the SDK's four-value enum does not cover. A
  // reader folding it onto a name would write a different file back.
  let material: ThmMaterial = ThmMaterial::from(6);

  assert_eq!(material, ThmMaterial::Unknown(6));
  assert_eq!(u32::from(material), 6);
  assert_eq!(material.label(), "6");
}

#[test]
fn serializes_an_engine_value_as_the_number_the_file_stores() {
  // The wire shape is a contract: `thm patch-bump` reports the previous mode under the 0011 envelope, and a surface
  // reading it expects the number the descriptor carries. A name would be a nicer report and a different one.
  let file: ThmFile = ThmFile {
    texture_type: Some(ThmTextureType::BumpMap),
    ..ThmFile::new_texture()
  };
  let json: Value = serde_json::to_value(&file).expect("Expect a descriptor to serialize");

  assert_eq!(json["textureType"], Value::from(2));
  assert_eq!(json["textureParam"]["format"], Value::from(0));
  assert_eq!(json["textureParam"]["flags"], Value::from(0x101));
  assert_eq!(json["textureParam"]["mipFilter"], Value::from(0));
  assert_eq!(json["material"]["material"], Value::from(1));
  assert_eq!(json["bump"]["mode"], Value::from(1));

  assert_eq!(
    serde_json::from_value::<ThmFile>(json).expect("Expect the numbers to read back"),
    file
  );
}

#[test]
fn reports_the_flag_bits_the_sdk_has_no_name_for() {
  // The word is not a set of twelve booleans: a file may carry bits nothing names, and a surface saying so is better
  // than one that silently shows them as absent.
  let flags: ThmTextureFlags = ThmTextureFlags::from(ThmTextureFlag::HasAlpha.bit() | 1 << 30);

  assert_eq!(flags.named().collect::<Vec<_>>(), vec![ThmTextureFlag::HasAlpha]);
  assert_eq!(flags.unnamed(), 1 << 30);
  assert_eq!(flags.raw(), ThmTextureFlag::HasAlpha.bit() | 1 << 30);
}

#[test]
fn answers_what_the_engine_reads_out_of_a_descriptor() {
  // The type gates everything else, so a complete bump chunk under a bump map type declares nothing to the engine.
  let mut file: ThmFile = ThmFile::new_texture();

  file.bump = Some(ThmBumpChunk {
    virtual_height: ThmBumpChunk::DEFAULT_VIRTUAL_HEIGHT,
    mode: ThmBumpMode::Use,
    name: String::from(fixtures::BUMP_NAME),
  });

  assert!(file.is_described_by_engine());
  assert_eq!(file.used_bump_name(), Some(fixtures::BUMP_NAME));

  file.texture_type = Some(ThmTextureType::BumpMap);

  assert!(!file.is_described_by_engine());
  assert_eq!(
    file.used_bump_name(),
    Some(fixtures::BUMP_NAME),
    "Expect the bump chunk to answer for itself, leaving the type gate to the caller"
  );
}

#[test]
fn answers_how_a_detail_association_is_applied() {
  // A name without either flag is dead data, and so is a flag without a name.
  let mut file: ThmFile = ThmFile::new_texture();

  assert_eq!(file.used_detail_usage(), None, "Expect no usage without a detail name");

  file.detail = Some(ThmDetailChunk {
    name: String::from("detail\\detail_grnd_grass"),
    scale: 1.0,
  });

  assert_eq!(file.used_detail_usage(), None, "Expect no usage without either flag");

  for (flag, expected) in [
    (ThmTextureFlag::DiffuseDetail, ThmDetailUsage::Diffuse),
    (ThmTextureFlag::BumpDetail, ThmDetailUsage::Bump),
  ] {
    let param: &mut ThmTextureParamChunk = file.texture_param.as_mut().expect("Expect texture params");

    param.flags = ThmTextureFlags::none().with(flag, true);

    assert_eq!(file.used_detail_usage(), Some(expected));
  }

  let param: &mut ThmTextureParamChunk = file.texture_param.as_mut().expect("Expect texture params");

  param.flags = ThmTextureFlags::none()
    .with(ThmTextureFlag::DiffuseDetail, true)
    .with(ThmTextureFlag::BumpDetail, true);

  assert_eq!(file.used_detail_usage(), Some(ThmDetailUsage::DiffuseAndBump));
}
