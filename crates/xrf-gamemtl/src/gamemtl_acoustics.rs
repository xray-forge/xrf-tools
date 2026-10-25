use byteorder::{ByteOrder, ReadBytesExt, WriteBytesExt};
use serde::{Deserialize, Serialize};
use xrf_chunk::{ChunkDataSource, ChunkReadWrite, ChunkReader, ChunkWriter};
use xrf_error::XrfResult;

/// How a material treats sound that reaches it, `SGameMtl::MtlAcoustics`.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GameMtlAcoustics {
  pub absorption: [f32; 3],
  pub scattering: f32,
  pub transmission: [f32; 3],
}

impl GameMtlAcoustics {
  /// What `assign_default_acoustics` (`xrMaterialSystem/GameMtlLib.cpp`) gives a material that declares none, by its
  /// exact name. The first entry is also the fallback for every name the table does not hold.
  pub const PREDEFINED: [(&'static str, Self); 50] = [
    ("default", Self::new([0.10, 0.20, 0.30], 0.05, [0.100, 0.050, 0.030])),
    (
      "default_object",
      Self::new([0.10, 0.20, 0.30], 0.05, [0.100, 0.050, 0.030]),
    ),
    (
      "materials\\asphalt",
      Self::new([0.15, 0.25, 0.35], 0.1, [0.05, 0.02, 0.01]),
    ),
    (
      "materials\\bricks",
      Self::new([0.03, 0.04, 0.07], 0.05, [0.015, 0.015, 0.015]),
    ),
    (
      "materials\\bush",
      Self::new([0.20, 0.30, 0.40], 0.2, [0.10, 0.05, 0.02]),
    ),
    (
      "materials\\bush_sux",
      Self::new([0.30, 0.40, 0.50], 0.3, [0.15, 0.07, 0.03]),
    ),
    (
      "materials\\cloth",
      Self::new([0.20, 0.30, 0.40], 0.2, [0.10, 0.05, 0.02]),
    ),
    (
      "materials\\concrete",
      Self::new([0.05, 0.07, 0.08], 0.05, [0.015, 0.002, 0.001]),
    ),
    ("materials\\death", Self::new([0.00, 0.00, 0.00], 0.00, [1.0, 1.0, 1.0])),
    (
      "materials\\dirt",
      Self::new([0.20, 0.30, 0.40], 0.15, [0.08, 0.04, 0.02]),
    ),
    (
      "materials\\earth",
      Self::new([0.25, 0.35, 0.45], 0.20, [0.10, 0.05, 0.03]),
    ),
    (
      "materials\\earth_death",
      Self::new([0.25, 0.35, 0.45], 0.20, [0.10, 0.05, 0.03]),
    ),
    (
      "materials\\earth_slide",
      Self::new([0.25, 0.35, 0.45], 0.20, [0.10, 0.05, 0.03]),
    ),
    (
      "materials\\flooring_tile",
      Self::new([0.01, 0.02, 0.02], 0.05, [0.060, 0.044, 0.011]),
    ),
    (
      "materials\\glass",
      Self::new([0.06, 0.03, 0.02], 0.05, [0.060, 0.044, 0.011]),
    ),
    (
      "materials\\grass",
      Self::new([0.30, 0.40, 0.50], 0.25, [0.12, 0.06, 0.04]),
    ),
    (
      "materials\\gravel",
      Self::new([0.60, 0.70, 0.80], 0.05, [0.031, 0.012, 0.008]),
    ),
    (
      "materials\\metal",
      Self::new([0.20, 0.07, 0.06], 0.05, [0.200, 0.025, 0.010]),
    ),
    (
      "materials\\metal_pipe",
      Self::new([0.15, 0.10, 0.08], 0.10, [0.10, 0.050, 0.020]),
    ),
    (
      "materials\\metal_plate",
      Self::new([0.10, 0.05, 0.04], 0.02, [0.30, 0.030, 0.015]),
    ),
    (
      "materials\\sand",
      Self::new([0.20, 0.30, 0.40], 0.2, [0.10, 0.05, 0.02]),
    ),
    (
      "materials\\setka_rabica",
      Self::new([0.01, 0.01, 0.01], 0.02, [0.5, 0.5, 0.5]),
    ),
    (
      "materials\\shifer",
      Self::new([0.02, 0.04, 0.06], 0.2, [0.05, 0.02, 0.01]),
    ),
    (
      "materials\\stucco",
      Self::new([0.01, 0.02, 0.02], 0.05, [0.060, 0.044, 0.011]),
    ),
    ("materials\\tin", Self::new([0.05, 0.10, 0.15], 0.2, [0.70, 0.60, 0.50])),
    (
      "materials\\tree_trunk",
      Self::new([0.11, 0.07, 0.06], 0.05, [0.070, 0.014, 0.005]),
    ),
    (
      "materials\\water",
      Self::new([0.01, 0.02, 0.03], 0.05, [0.99, 0.98, 0.97]),
    ),
    (
      "materials\\water_radiation",
      Self::new([0.01, 0.02, 0.03], 0.05, [0.99, 0.98, 0.97]),
    ),
    (
      "materials\\wood",
      Self::new([0.11, 0.07, 0.06], 0.05, [0.070, 0.014, 0.005]),
    ),
    (
      "materials\\wooden_board",
      Self::new([0.11, 0.07, 0.06], 0.05, [0.070, 0.014, 0.005]),
    ),
    (
      "objects\\barrel",
      Self::new([0.20, 0.30, 0.40], 0.2, [0.05, 0.03, 0.02]),
    ),
    (
      "objects\\bottle",
      Self::new([0.06, 0.03, 0.02], 0.05, [0.06, 0.04, 0.01]),
    ),
    (
      "objects\\bullet",
      Self::new([0.20, 0.07, 0.06], 0.05, [0.200, 0.025, 0.010]),
    ),
    (
      "objects\\car_cabine",
      Self::new([0.2, 0.3, 0.4], 0.2, [0.1, 0.05, 0.02]),
    ),
    ("objects\\car_wheel", Self::new([0.2, 0.3, 0.4], 0.2, [0.1, 0.05, 0.02])),
    (
      "objects\\clothes",
      Self::new([0.25, 0.35, 0.45], 0.3, [0.15, 0.07, 0.03]),
    ),
    (
      "objects\\concrete_box",
      Self::new([0.05, 0.07, 0.08], 0.05, [0.015, 0.002, 0.001]),
    ),
    (
      "objects\\dead_body",
      Self::new([0.10, 0.20, 0.30], 0.05, [0.100, 0.050, 0.030]),
    ),
    ("objects\\fuel_can", Self::new([0.3, 0.4, 0.5], 0.2, [0.1, 0.05, 0.02])),
    (
      "objects\\glass",
      Self::new([0.06, 0.03, 0.02], 0.05, [0.060, 0.044, 0.011]),
    ),
    (
      "objects\\knife",
      Self::new([0.20, 0.07, 0.06], 0.05, [0.200, 0.025, 0.010]),
    ),
    (
      "objects\\large_furniture",
      Self::new([0.20, 0.30, 0.40], 0.2, [0.05, 0.03, 0.02]),
    ),
    (
      "objects\\large_metal_trash",
      Self::new([0.20, 0.07, 0.06], 0.05, [0.200, 0.025, 0.010]),
    ),
    (
      "objects\\large_weapon",
      Self::new([0.20, 0.07, 0.06], 0.05, [0.200, 0.025, 0.010]),
    ),
    (
      "objects\\metal_box",
      Self::new([0.20, 0.07, 0.06], 0.05, [0.200, 0.025, 0.010]),
    ),
    (
      "objects\\monster_body",
      Self::new([0.6, 0.7, 0.8], 0.3, [0.1, 0.05, 0.02]),
    ),
    (
      "objects\\small_box",
      Self::new([0.10, 0.05, 0.04], 0.02, [0.30, 0.030, 0.015]),
    ),
    (
      "objects\\small_metal_trash",
      Self::new([0.10, 0.05, 0.04], 0.02, [0.30, 0.030, 0.015]),
    ),
    (
      "objects\\small_weapon",
      Self::new([0.10, 0.05, 0.04], 0.02, [0.30, 0.030, 0.015]),
    ),
    (
      "objects\\tin_can",
      Self::new([0.05, 0.10, 0.15], 0.2, [0.70, 0.60, 0.50]),
    ),
  ];

  pub const fn new(absorption: [f32; 3], scattering: f32, transmission: [f32; 3]) -> Self {
    Self {
      absorption,
      scattering,
      transmission,
    }
  }

  /// What the engine gives a material declaring no acoustics: its predefined entry by exact name, or else the
  /// `default` entry absorbing as much as the material occludes.
  pub fn get_default(name: &str, sound_occlusion_factor: f32) -> Self {
    match Self::PREDEFINED.iter().find(|(predefined, _)| *predefined == name) {
      Some((_, acoustics)) => *acoustics,
      None => Self {
        absorption: [sound_occlusion_factor; 3],
        ..Self::PREDEFINED[0].1
      },
    }
  }
}

impl ChunkReadWrite for GameMtlAcoustics {
  fn read<T: ByteOrder, D: ChunkDataSource>(reader: &mut ChunkReader<D>) -> XrfResult<Self> {
    Ok(Self {
      absorption: [
        reader.read_f32::<T>()?,
        reader.read_f32::<T>()?,
        reader.read_f32::<T>()?,
      ],
      scattering: reader.read_f32::<T>()?,
      transmission: [
        reader.read_f32::<T>()?,
        reader.read_f32::<T>()?,
        reader.read_f32::<T>()?,
      ],
    })
  }

  fn write<T: ByteOrder>(&self, writer: &mut ChunkWriter) -> XrfResult {
    for value in self.absorption {
      writer.write_f32::<T>(value)?;
    }

    writer.write_f32::<T>(self.scattering)?;

    for value in self.transmission {
      writer.write_f32::<T>(value)?;
    }

    Ok(())
  }
}
