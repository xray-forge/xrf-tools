/** How the surfaces of a level are drawn while a toggle is on. */
export interface ILevelSurfaceOptions {
  isWireframe: boolean;
  /** Draws the surfaces with the textures the level dresses them in, or flat for comparison. */
  isTextured: boolean;
  /**
   * Whether the hemisphere occlusion xrLC baked into the level is applied, or the level is drawn under the viewer's
   * own light alone.
   */
  isLit: boolean;
}

export const DEFAULT_LEVEL_SURFACE_OPTIONS: ILevelSurfaceOptions = {
  isLit: true,
  isTextured: true,
  isWireframe: false,
};
