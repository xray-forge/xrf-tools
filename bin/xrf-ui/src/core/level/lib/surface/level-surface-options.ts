/** How the surfaces of a level are drawn while a toggle is on. */
export interface ILevelSurfaceOptions {
  isWireframe: boolean;
  /** Draws the surfaces with the textures the level dresses them in, or each in its own colour for comparison. */
  isTextured: boolean;
  /**
   * Whether the hemisphere occlusion xrLC baked into the level is applied, or the level is drawn under the viewer's
   * own light alone.
   */
  isBaked: boolean;
  /** Whether surfaces shade with the bump pairs their base textures declare, or flat for comparison. */
  isBumped: boolean;
}

export const DEFAULT_LEVEL_SURFACE_OPTIONS: ILevelSurfaceOptions = {
  isBaked: true,
  isBumped: true,
  isTextured: true,
  isWireframe: false,
};
