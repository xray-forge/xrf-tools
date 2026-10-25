use byteorder::{ByteOrder, ReadBytesExt, WriteBytesExt};
use serde::{Deserialize, Serialize};
use xrf_chunk::{ChunkDataSource, ChunkReadWrite, ChunkReader, ChunkWriter};
use xrf_error::XrfResult;

use crate::gamemtl_acoustics::GameMtlAcoustics;
use crate::gamemtl_version::GameMtlVersion;

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
  /// How freely a bullet passes, from 0 to 1 where 1 is straight through.
  pub shoot_factor: f32,
  pub bounce_damage_factor: f32,
  /// How much it is seen through, from 0 to 1 where 1 is fully transparent.
  pub visual_transparency_factor: f32,
  /// How much sound it lets through, from 0 to 1 where 1 is fully heard.
  pub sound_occlusion_factor: f32,
  /// How freely the material is walked through; below one the engine treats it as slowing the walker down.
  pub flotation_factor: Option<f32>,
  /// How fast standing in it costs health, which is what makes a material injurious.
  pub injurious_speed: Option<f32>,
  pub density_factor: Option<f32>,
  /// The multiplayer shoot factor.
  pub shoot_factor_mp: Option<f32>,
  /// Absent in every shipped library; the engine then takes a predefined set by name.
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

  pub const FLAG_BREAKABLE: u32 = 1 << 0;
  pub const FLAG_BOUNCEABLE: u32 = 1 << 2;
  pub const FLAG_SKIDMARK: u32 = 1 << 3;
  pub const FLAG_BLOODMARK: u32 = 1 << 4;
  pub const FLAG_CLIMABLE: u32 = 1 << 5;
  /// Things walk and fall through the material, as grass planted over it does.
  pub const FLAG_PASSABLE: u32 = 1 << 7;
  /// Set on what moves; a pair exists only where at least one of its materials is dynamic.
  pub const FLAG_DYNAMIC: u32 = 1 << 8;
  pub const FLAG_LIQUID: u32 = 1 << 9;
  pub const FLAG_SUPPRESS_SHADOWS: u32 = 1 << 10;
  pub const FLAG_SUPPRESS_WALLMARKS: u32 = 1 << 11;
  pub const FLAG_ACTOR_OBSTACLE: u32 = 1 << 12;
  pub const FLAG_NO_RICOCHET: u32 = 1 << 13;
  /// The editor sets it on save where the injurious speed is not zero.
  pub const FLAG_INJURIOUS: u32 = 1 << 28;
  /// The editor sets it on save where the shoot factor is zero.
  pub const FLAG_SHOOTABLE: u32 = 1 << 29;
  /// The editor sets it on save where the visual transparency factor is zero.
  pub const FLAG_TRANSPARENT: u32 = 1 << 30;
  /// The editor sets it on save where the flotation factor is not one.
  pub const FLAG_SLOW_DOWN: u32 = 1 << 31;

  /// Every flag the engine names, in bit order. Bits 1 and 6 are the retired shootable and walk-on flags, which shipped
  /// libraries still set and nothing reads.
  pub const FLAG_NAMES: [(u32, &'static str); 16] = [
    (Self::FLAG_BREAKABLE, "breakable"),
    (Self::FLAG_BOUNCEABLE, "bounceable"),
    (Self::FLAG_SKIDMARK, "skidmark"),
    (Self::FLAG_BLOODMARK, "bloodmark"),
    (Self::FLAG_CLIMABLE, "climable"),
    (Self::FLAG_PASSABLE, "passable"),
    (Self::FLAG_DYNAMIC, "dynamic"),
    (Self::FLAG_LIQUID, "liquid"),
    (Self::FLAG_SUPPRESS_SHADOWS, "suppress shadows"),
    (Self::FLAG_SUPPRESS_WALLMARKS, "suppress wallmarks"),
    (Self::FLAG_ACTOR_OBSTACLE, "actor obstacle"),
    (Self::FLAG_NO_RICOCHET, "no ricochet"),
    (Self::FLAG_INJURIOUS, "injurious"),
    (Self::FLAG_SHOOTABLE, "shootable"),
    (Self::FLAG_TRANSPARENT, "transparent"),
    (Self::FLAG_SLOW_DOWN, "slow down"),
  ];

  /// What `SGameMtl` holds for flotation where the chunk is absent.
  pub const DEFAULT_FLOTATION_FACTOR: f32 = 1.0;
  /// What `SGameMtl` holds for the injurious speed where the chunk is absent.
  pub const DEFAULT_INJURIOUS_SPEED: f32 = 0.0;
  /// What `SGameMtl` holds for density where the chunk is absent.
  pub const DEFAULT_DENSITY_FACTOR: f32 = 0.0;

  /// Reads one material from the chunk holding it.
  ///
  /// # Errors
  ///
  /// Returns an error when a chunk the engine asserts on is absent, or a chunk holds more or less than it should.
  pub fn read<T: ByteOrder, D: ChunkDataSource>(reader: &mut ChunkReader<D>) -> XrfResult<Self> {
    let chunks: Vec<ChunkReader<D>> = reader.read_children()?;

    let mut main: ChunkReader<D> = ChunkReader::find_required_by_id(&chunks, Self::MAIN_CHUNK_ID)?;
    let id: u32 = main.read_u32::<T>()?;
    let name: String = main.read_w1251_string()?;

    main.assert_read("All data should be read in game material main chunk")?;

    let mut physics: ChunkReader<D> = ChunkReader::find_required_by_id(&chunks, Self::PHYSICS_CHUNK_ID)?;
    let mut factors: ChunkReader<D> = ChunkReader::find_required_by_id(&chunks, Self::FACTORS_CHUNK_ID)?;

    let material: Self = Self {
      id,
      name,
      description: ChunkReader::find_optional_by_id(&chunks, Self::DESCRIPTION_CHUNK_ID)
        .map(|mut chunk| chunk.read_w1251_string_payload())
        .transpose()?,
      flags: ChunkReader::find_required_by_id(&chunks, Self::FLAGS_CHUNK_ID)?.read_u32_payload::<T>()?,
      ph_friction: physics.read_f32::<T>()?,
      ph_damping: physics.read_f32::<T>()?,
      ph_spring: physics.read_f32::<T>()?,
      ph_bounce_start_velocity: physics.read_f32::<T>()?,
      ph_bouncing: physics.read_f32::<T>()?,
      shoot_factor: factors.read_f32::<T>()?,
      bounce_damage_factor: factors.read_f32::<T>()?,
      visual_transparency_factor: factors.read_f32::<T>()?,
      sound_occlusion_factor: factors.read_f32::<T>()?,
      flotation_factor: Self::read_optional_f32::<T, D>(&chunks, Self::FLOTATION_CHUNK_ID)?,
      injurious_speed: Self::read_optional_f32::<T, D>(&chunks, Self::INJURIOUS_CHUNK_ID)?,
      density_factor: Self::read_optional_f32::<T, D>(&chunks, Self::DENSITY_CHUNK_ID)?,
      shoot_factor_mp: Self::read_optional_f32::<T, D>(&chunks, Self::FACTORS_MP_CHUNK_ID)?,
      acoustics: Self::read_optional_acoustics::<T, D>(&chunks)?,
    };

    physics.assert_read("All data should be read in game material physics chunk")?;
    factors.assert_read("All data should be read in game material factors chunk")?;

    Ok(material)
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

    // `SGameMtl::Save` puts the multiplayer factor before the three chunks that follow it, which is not the order
    // `SGameMtl::Load` reads them in; keeping the editor's order is what writes a shipped library back byte for byte.
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

  /// Reads one optional single-float chunk, or nothing where the material declares none.
  fn read_optional_f32<T: ByteOrder, D: ChunkDataSource>(chunks: &[ChunkReader<D>], id: u32) -> XrfResult<Option<f32>> {
    ChunkReader::find_optional_by_id(chunks, id)
      .map(|mut chunk| chunk.read_f32_payload::<T>())
      .transpose()
  }

  /// Reads the acoustics chunk, or nothing where the material declares none.
  fn read_optional_acoustics<T: ByteOrder, D: ChunkDataSource>(
    chunks: &[ChunkReader<D>],
  ) -> XrfResult<Option<GameMtlAcoustics>> {
    let Some(mut chunk) = ChunkReader::find_optional_by_id(chunks, Self::ACOUSTICS_CHUNK_ID) else {
      return Ok(None);
    };
    let acoustics: GameMtlAcoustics = GameMtlAcoustics::read::<T, D>(&mut chunk)?;

    chunk.assert_read("All data should be read in game material acoustics chunk")?;

    Ok(Some(acoustics))
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

  /// Whether things walk and fall through it, as grass planted over it does.
  pub const fn is_passable(&self) -> bool {
    self.flags & Self::FLAG_PASSABLE != 0
  }

  /// The flags the material sets, named.
  pub fn get_named_flags(&self) -> Vec<&'static str> {
    Self::FLAG_NAMES
      .into_iter()
      .filter_map(|(mask, name)| (self.flags & mask != 0).then_some(name))
      .collect()
  }

  /// The flotation factor the engine plays with, its default where the material declares none.
  pub fn get_flotation_factor(&self) -> f32 {
    self.flotation_factor.unwrap_or(Self::DEFAULT_FLOTATION_FACTOR)
  }

  /// The injurious speed the engine plays with, its default where the material declares none.
  pub fn get_injurious_speed(&self) -> f32 {
    self.injurious_speed.unwrap_or(Self::DEFAULT_INJURIOUS_SPEED)
  }

  /// The density factor the engine plays with, its default where the material declares none.
  pub fn get_density_factor(&self) -> f32 {
    self.density_factor.unwrap_or(Self::DEFAULT_DENSITY_FACTOR)
  }

  /// The multiplayer shoot factor the engine plays with, the single-player one where the material declares none.
  pub fn get_shoot_factor_mp(&self) -> f32 {
    self.shoot_factor_mp.unwrap_or(self.shoot_factor)
  }

  /// The acoustics the engine plays with: the declared ones, or the default `GameMtlAcoustics` gives the name.
  pub fn get_acoustics(&self) -> GameMtlAcoustics {
    self
      .acoustics
      .unwrap_or_else(|| GameMtlAcoustics::get_default(&self.name, self.sound_occlusion_factor))
  }

  /// The game `SGameMtl::Load` takes the material for, by the newest chunk it carries.
  pub const fn get_game_version(&self) -> GameMtlVersion {
    if self.shoot_factor_mp.is_some() {
      GameMtlVersion::CallOfPripyat
    } else if self.density_factor.is_some() {
      GameMtlVersion::ClearSky
    } else {
      GameMtlVersion::ShadowOfChernobyl
    }
  }
}
