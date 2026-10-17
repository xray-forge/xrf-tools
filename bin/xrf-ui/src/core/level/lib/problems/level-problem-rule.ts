/**
 * What a level's problems are grouped by, which is the stage of the open each one comes from.
 */
export enum ELevelProblemRule {
  /** A texture reference of the level's surfaces the roots could not answer properly. */
  TEXTURE = "texture",
  /** A texture reference only the grass samples that the roots could not answer properly. */
  GRASS_TEXTURE = "grass-texture",
  /** A shader table entry the library could not be read for, so the surface is drawn as an unresolved one. */
  SURFACE = "surface",
  /** A drawable the packer could not read, so nothing of it is drawn at all. */
  DRAWABLE = "drawable",
  /** A visual spawned objects name that could not be read, so none of them is drawn. */
  SPAWN = "spawn",
}
