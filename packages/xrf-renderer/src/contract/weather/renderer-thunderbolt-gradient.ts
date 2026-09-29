import { ERendererDraw } from "#/contract/scene/renderer-draw";

/**
 * A glow a bolt draws facing the view (`SThunderboltDesc::SFlare`): at its top, or at its middle.
 */
export interface IRendererThunderboltGradient {
  /** Times the strike's phase. */
  opacity: number;
  /** Across and up, as fractions of the bolt's length. */
  radius: readonly [number, number];
  /** The texture's reference, a key of the weather's textures. */
  texture: string;
  /** How its shader composites it. */
  draw: ERendererDraw;
}
