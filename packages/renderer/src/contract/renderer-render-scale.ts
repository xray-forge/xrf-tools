/**
 * How much smaller than the output the scene is drawn and then upscaled: FSR's quality modes, by the ratio of the
 * output's side to the drawing's. TAA upscales in its resolve, FSR in its own, and the other modes with FSR 1.
 */
export enum ERendererRenderScale {
  NATIVE = "native",
  /** 1.5: two thirds of each side. */
  QUALITY = "quality",
  /** 1.7. */
  BALANCED = "balanced",
  /** 2: half of each side. */
  PERFORMANCE = "performance",
}

/** Each render scale's ratio of the output's side to the drawing's. */
export const RENDERER_RENDER_SCALE_RATIOS: Readonly<Record<ERendererRenderScale, number>> = {
  [ERendererRenderScale.NATIVE]: 1,
  [ERendererRenderScale.QUALITY]: 1.5,
  [ERendererRenderScale.BALANCED]: 1.7,
  [ERendererRenderScale.PERFORMANCE]: 2,
};
