/// Which game's scripts an object's script class follows, Clear Sky's or Call of Pripyat's.
///
/// The two games register different script classes for some sections (`class_registrator.script`) and save different
/// fields from the ones they share (`se_stalker`, `se_monster`, `se_smart_terrain`, `se_actor`). The script version
/// cannot tell them apart, so the dialect follows the spawn version an object's packet stores, never the game a tool is
/// pointed at: a Call of Pripyat mod can ship Clear Sky's levels.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ScriptDialect {
  ClearSky,
  CallOfPripyat,
}
