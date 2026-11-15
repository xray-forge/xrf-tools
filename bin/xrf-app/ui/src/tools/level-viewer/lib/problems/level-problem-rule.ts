/**
 * What a level's problems past its textures' are grouped by, which is the stage of the open each one comes from; the
 * textures' are `ELevelTextureProblemRule`.
 */
export enum ELevelProblemRule {
  /** A shader table entry the library could not be read for, so the surface is drawn as an unresolved one. */
  SURFACE = "surface",
  /** A drawable the packer could not read, so nothing of it is drawn at all. */
  DRAWABLE = "drawable",
  /** A visual spawned objects name that could not be read, so none of them is drawn. */
  SPAWN = "spawn",
}
