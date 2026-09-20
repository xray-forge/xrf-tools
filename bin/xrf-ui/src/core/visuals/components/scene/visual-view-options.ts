/**
 * View state the toolbar owns and the scene applies.
 */
export interface IVisualPreviewViewOptions {
  isWireframe: boolean;
  isGridVisible: boolean;
  isAxesVisible: boolean;
  /** Renders a repeating checkerboard from the uv buffer instead of a flat surface. */
  isCheckerVisible: boolean;
  /** Draws the bind pose over the mesh. */
  isSkeletonVisible: boolean;
  /** Shades bumped materials the way the game does, or draws them flat. */
  isBumpVisible: boolean;
  /** Cuts out and blends the surfaces whose shader reads alpha, the way the game does, or draws them solid. */
  isAlphaVisible: boolean;
}

/**
 * How a preview looks before anyone touches a toggle.
 */
export const DEFAULT_VISUAL_PREVIEW_VIEW_OPTIONS: IVisualPreviewViewOptions = {
  isWireframe: false,
  isGridVisible: true,
  isAxesVisible: true,
  isCheckerVisible: false,
  isSkeletonVisible: false,
  isBumpVisible: true,
  isAlphaVisible: true,
};
