use byteorder::{ByteOrder, ReadBytesExt, WriteBytesExt};
use serde::{Deserialize, Serialize};
use xrf_chunk::{
  ChunkDataSource, ChunkReadWrite, ChunkReader, ChunkWriter, find_optional_chunk_by_id, find_required_chunk_by_id,
  read_f32_chunk, read_u32_chunk,
};
use xrf_error::XrfResult;

use crate::gamemtl::gamemtl_acoustics::GameMtlAcoustics;

/// Flags a material carries, by the bit `SGameMtl` names it under.
pub const GAMEMTL_FLAGS: [(u32, &str); 14] = [
  (1 << 0, "breakable"),
  (1 << 2, "bounceable"),
  (1 << 3, "skidmark"),
  (1 << 4, "bloodmark"),
  (1 << 5, "climable"),
  (1 << 7, "passable"),
  (1 << 8, "dynamic"),
  (1 << 9, "liquid"),
  (1 << 10, "suppress shadows"),
  (1 << 11, "suppress wallmarks"),
  (1 << 12, "actor obstacle"),
  (1 << 13, "no ricochet"),
  (1 << 28, "injurious"),
  (1 << 29, "shootable"),
];

/// One game material, `SGameMtl` (`xrMaterialSystem/GameMtlLib.cpp`).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GameMtlMaterial {
  /// The library's own number for the material, which a collision face stores rather than the name.
  pub id: u32,
  pub name: String,
  /// Absent where the material declares no description chunk at all.
  pub description: Option<String>,
  pub flags: u32,
  pub ph_friction: f32,
  pub ph_damping: f32,
  pub ph_spring: f32,
  pub ph_bounce_start_velocity: f32,
  pub ph_bouncing: f32,
  pub shoot_factor: f32,
  pub bounce_damage_factor: f32,
  pub visual_transparency_factor: f32,
  pub sound_occlusion_factor: f32,
  /// How freely the material is walked through; below one the engine treats it as slowing the walker down.
  pub flotation_factor: Option<f32>,
  /// How fast standing in it costs health, which is what makes a material injurious.
  pub injurious_speed: Option<f32>,
  pub density_factor: Option<f32>,
  /// The multiplayer shoot factor, which the engine falls back to the single-player one for when absent.
  pub shoot_factor_mp: Option<f32>,
  /// Absent where the material predates the chunk; the engine then derives it from the occlusion factor.
  pub acoustics: Option<GameMtlAcoustics>,
}

impl GameMtlMaterial {
  pub const MAIN_CHUNK_ID: u32 = 0x1000;
  pub const FLAGS_CHUNK_ID: u32 = 0x1001;
  pub const PHYSICS_CHUNK_ID: u32 = 0x1002;
  pub const FACTORS_CHUNK_ID: u32 = 0x1003;
  pub const FLOTATION_CHUNK_ID: u32 = 0x1004;
  pub const DESCRIPTION_CHUNK_ID: u32 = 0x1005;
  pub const INJURIOUS_CHUNK_ID: u32 = 0x1006;
  pub const DENSITY_CHUNK_ID: u32 = 0x1007;
  pub const FACTORS_MP_CHUNK_ID: u32 = 0x1008;
  pub const ACOUSTICS_CHUNK_ID: u32 = 0x1009;

  /// Reads one material from the chunk holding it.
  ///
  /// # Errors
  ///
  /// Returns an error when a chunk the engine asserts on is absent, or a chunk does not hold what it should.
  pub fn read<T: ByteOrder, D: ChunkDataSource>(reader: &mut ChunkReader<D>) -> XrfResult<Self> {
    let chunks: Vec<ChunkReader<D>> = reader.read_children()?;

    let mut main: ChunkReader<D> = find_required_chunk_by_id(&chunks, Self::MAIN_CHUNK_ID)?;
    let id: u32 = main.read_u32::<T>()?;
    let name: String = main.read_w1251_string()?;

    let mut physics: ChunkReader<D> = find_required_chunk_by_id(&chunks, Self::PHYSICS_CHUNK_ID)?;
    let mut factors: ChunkReader<D> = find_required_chunk_by_id(&chunks, Self::FACTORS_CHUNK_ID)?;

    Ok(Self {
      id,
      name,
      description: find_optional_chunk_by_id(&chunks, Self::DESCRIPTION_CHUNK_ID)
        .map(|mut chunk| chunk.read_w1251_string())
        .transpose()?,
      flags: read_u32_chunk::<T, D>(&mut find_required_chunk_by_id(&chunks, Self::FLAGS_CHUNK_ID)?)?,
      ph_friction: physics.read_f32::<T>()?,
      ph_damping: physics.read_f32::<T>()?,
      ph_spring: physics.read_f32::<T>()?,
      ph_bounce_start_velocity: physics.read_f32::<T>()?,
      ph_bouncing: physics.read_f32::<T>()?,
      shoot_factor: factors.read_f32::<T>()?,
      bounce_damage_factor: factors.read_f32::<T>()?,
      visual_transparency_factor: factors.read_f32::<T>()?,
      sound_occlusion_factor: factors.read_f32::<T>()?,
      flotation_factor: find_optional_chunk_by_id(&chunks, Self::FLOTATION_CHUNK_ID)
        .map(|mut chunk| read_f32_chunk::<T, D>(&mut chunk))
        .transpose()?,
      injurious_speed: find_optional_chunk_by_id(&chunks, Self::INJURIOUS_CHUNK_ID)
        .map(|mut chunk| read_f32_chunk::<T, D>(&mut chunk))
        .transpose()?,
      density_factor: find_optional_chunk_by_id(&chunks, Self::DENSITY_CHUNK_ID)
        .map(|mut chunk| read_f32_chunk::<T, D>(&mut chunk))
        .transpose()?,
      shoot_factor_mp: find_optional_chunk_by_id(&chunks, Self::FACTORS_MP_CHUNK_ID)
        .map(|mut chunk| read_f32_chunk::<T, D>(&mut chunk))
        .transpose()?,
      acoustics: find_optional_chunk_by_id(&chunks, Self::ACOUSTICS_CHUNK_ID)
        .map(|mut chunk| GameMtlAcoustics::read::<T, D>(&mut chunk))
        .transpose()?,
    })
  }

  /// Writes the material back in the chunk order the editor wrote it in.
  ///
  /// # Errors
  ///
  /// Returns an error when the writer refuses the bytes.
  pub fn write<T: ByteOrder>(&self, writer: &mut ChunkWriter) -> XrfResult {
    let mut main: ChunkWriter = ChunkWriter::new();

    main.write_u32::<T>(self.id)?;
    main.write_w1251_string(&self.name)?;
    main.flush_chunk_into::<T>(&mut writer.buffer, Self::MAIN_CHUNK_ID)?;

    if let Some(description) = &self.description {
      let mut chunk: ChunkWriter = ChunkWriter::new();

      chunk.write_w1251_string(description)?;
      chunk.flush_chunk_into::<T>(&mut writer.buffer, Self::DESCRIPTION_CHUNK_ID)?;
    }

    let mut flags: ChunkWriter = ChunkWriter::new();

    flags.write_u32::<T>(self.flags)?;
    flags.flush_chunk_into::<T>(&mut writer.buffer, Self::FLAGS_CHUNK_ID)?;

    let mut physics: ChunkWriter = ChunkWriter::new();

    physics.write_f32::<T>(self.ph_friction)?;
    physics.write_f32::<T>(self.ph_damping)?;
    physics.write_f32::<T>(self.ph_spring)?;
    physics.write_f32::<T>(self.ph_bounce_start_velocity)?;
    physics.write_f32::<T>(self.ph_bouncing)?;
    physics.flush_chunk_into::<T>(&mut writer.buffer, Self::PHYSICS_CHUNK_ID)?;

    let mut factors: ChunkWriter = ChunkWriter::new();

    factors.write_f32::<T>(self.shoot_factor)?;
    factors.write_f32::<T>(self.bounce_damage_factor)?;
    factors.write_f32::<T>(self.visual_transparency_factor)?;
    factors.write_f32::<T>(self.sound_occlusion_factor)?;
    factors.flush_chunk_into::<T>(&mut writer.buffer, Self::FACTORS_CHUNK_ID)?;

    // The multiplayer factor goes before the three chunks that follow it, which is not the order
    // `SGameMtl::Load` reads them in and is the order every shipped library writes them in: 317 of the 377 materials
    // in the trees are `1000 1005 1001 1002 1003 1008 1004 1006 1007`, and the other 60 are the same without 1008.
    Self::write_optional_f32::<T>(writer, Self::FACTORS_MP_CHUNK_ID, self.shoot_factor_mp)?;
    Self::write_optional_f32::<T>(writer, Self::FLOTATION_CHUNK_ID, self.flotation_factor)?;
    Self::write_optional_f32::<T>(writer, Self::INJURIOUS_CHUNK_ID, self.injurious_speed)?;
    Self::write_optional_f32::<T>(writer, Self::DENSITY_CHUNK_ID, self.density_factor)?;

    if let Some(acoustics) = &self.acoustics {
      let mut chunk: ChunkWriter = ChunkWriter::new();

      acoustics.write::<T>(&mut chunk)?;
      chunk.flush_chunk_into::<T>(&mut writer.buffer, Self::ACOUSTICS_CHUNK_ID)?;
    }

    Ok(())
  }

  /// Writes one optional single-float chunk, or nothing where the material declares none.
  fn write_optional_f32<T: ByteOrder>(writer: &mut ChunkWriter, id: u32, value: Option<f32>) -> XrfResult {
    if let Some(value) = value {
      let mut chunk: ChunkWriter = ChunkWriter::new();

      chunk.write_f32::<T>(value)?;
      chunk.flush_chunk_into::<T>(&mut writer.buffer, id)?;
    }

    Ok(())
  }

  /// The flags the material sets, named.
  pub fn get_named_flags(&self) -> Vec<&'static str> {
    GAMEMTL_FLAGS
      .into_iter()
      .filter_map(|(mask, name)| (self.flags & mask != 0).then_some(name))
      .collect()
  }
}
