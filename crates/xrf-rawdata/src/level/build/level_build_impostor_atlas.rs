/// The impostor atlas a build writes beside `build.prj`, as the level ships it: `level_lods.dds` and
/// `level_lods_nm.dds`, DXT5 with mips.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LevelBuildImpostorAtlas {
  pub color: Vec<u8>,
  pub normal: Vec<u8>,
}
