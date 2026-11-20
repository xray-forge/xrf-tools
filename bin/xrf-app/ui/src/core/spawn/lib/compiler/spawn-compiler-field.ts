/**
 * The spawn compiler's path fields by the id each is remembered under: its form reads them, and a tool handing built
 * levels over to it writes them, so a renamed field cannot leave a hand-over filling one nothing reads.
 */
export enum ESpawnCompilerField {
  /** The game data holding `configs/game_levels.ltx`. */
  GAMEDATA = "gamedata",
  /** The directory holding the built levels' directories. */
  LEVELS_ROOT = "levels-root",
  /** The directory the spawn is written into. */
  DESTINATION = "destination",
}
