/**
 * What a level's problems are grouped by, which is the stage of the open each one comes from.
 */
export enum ELevelProblemRule {
  /** A texture reference the roots could not answer properly. */
  TEXTURE = "texture",
  /** A shader table entry the library could not be read for, so the surface is drawn as an unresolved one. */
  SURFACE = "surface",
  /** A drawable the packer could not read, so nothing of it is drawn at all. */
  DRAWABLE = "drawable",
}
