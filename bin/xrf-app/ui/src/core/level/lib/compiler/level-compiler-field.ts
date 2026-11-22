/**
 * The level compiler's path fields by the id each is remembered under: its form reads them, and a tool handing a scene
 * over to it writes them, so a renamed field cannot leave a hand-over filling one nothing reads.
 */
export enum ELevelCompilerField {
  /** The scene's `.level`. */
  SOURCE = "source",
  /** The level directory the build writes into. */
  DESTINATION = "destination",
  /** The game material library, `gamemtl.xr`. */
  GAME_MATERIALS = "game-materials",
  /** The textures root read before the game data's own. */
  TEXTURES = "textures",
  /** The objects roots the scene's objects are read from, a list kept through `PATH_LIST_CODEC`. */
  OBJECTS = "objects",
  /** The game data the compilers read their libraries and textures from. */
  GAMEDATA = "gamedata",
}
