import { ERendererAmbientOcclusionQuality } from "#/contract/renderer-ambient-occlusion-quality";

/**
 * Ambient occlusion from the depth of the frame, GTAO as XeGTAO computes it at half resolution: it darkens the
 * hemisphere and ambient light over the baked hemisphere occlusion, as the engine's SSAO does.
 */
export interface IRendererAmbientOcclusionSettings {
  isEnabled: boolean;
  /** Metres around a point that what stands there occludes it from. */
  radius: number;
  /** How dark the occlusion goes: one XeGTAO's own curve, zero none, two its square. */
  strength: number;
  quality: ERendererAmbientOcclusionQuality;
}

/** XeGTAO's defaults, at a metre. */
export const DEFAULT_RENDERER_AMBIENT_OCCLUSION_SETTINGS: IRendererAmbientOcclusionSettings = {
  isEnabled: true,
  quality: ERendererAmbientOcclusionQuality.HIGH,
  radius: 1,
  strength: 1,
};
