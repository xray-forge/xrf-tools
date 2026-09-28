/**
 * The engine's exposure (`r2_tonemap`): the scene's average luminance measured every frame, and the scale the tonemap
 * multiplies by moved towards `middle_gray / luminance` at the adaptation's rate
 * (`xrRender_R2/r2_rendertarget_phase_luminance.cpp`, `bloom_luminance_3.ps`). OpenXRay's defaults
 * (`xrRender_console.cpp`).
 */
export interface IRendererExposureSettings {
  /** `r2_tonemap`: off, the tonemap multiplies by one, the engine's answer at noon. */
  isEnabled: boolean;
  /** `r2_tonemap_amount`: how far from no adaptation towards the whole of it. */
  amount: number;
  /** `r2_tonemap_middlegray`: the luminance the frame is brought towards. */
  middleGray: number;
  /** `r2_tonemap_lowlum`: what the luminance is floored at, so a black frame is not brightened without end. */
  lowLuminance: number;
  /** `r2_tonemap_adaptation`: how fast the scale follows the frame. */
  adaptation: number;
}

/** OpenXRay's own exposure. */
export const DEFAULT_RENDERER_EXPOSURE_SETTINGS: IRendererExposureSettings = {
  adaptation: 1,
  amount: 0.7,
  isEnabled: true,
  lowLuminance: 0.0001,
  middleGray: 1,
};
