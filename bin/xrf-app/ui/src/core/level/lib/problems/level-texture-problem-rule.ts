/**
 * What a texture problem is grouped by: which part of what is drawn samples the texture.
 */
export enum ELevelTextureProblemRule {
  /** A texture reference of the surfaces drawn the roots could not answer properly. */
  TEXTURE = "texture",
  /** A texture reference only the grass samples that the roots could not answer properly. */
  GRASS_TEXTURE = "grass-texture",
}
