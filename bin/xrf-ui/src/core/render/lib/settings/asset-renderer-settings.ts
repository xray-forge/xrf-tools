import { ERendererDebugView, IRendererFeatureSettings, IRendererSettings } from "@xrf/renderer";

/**
 * The renderer's settings for one asset against a backdrop, which the texture and visual viewers both draw.
 *
 * @param view - What the viewer decides: its backdrop, its pacing and its toggles.
 * @param features - What the renderer's features are set to.
 * @returns The renderer's settings.
 */
export function toAssetRendererSettings(
  view: Pick<IRendererSettings, "backdrop" | "isBumped" | "isLit" | "isWireframe" | "pacing">,
  features: IRendererFeatureSettings
): IRendererSettings {
  return {
    ...view,
    debugView: ERendererDebugView.FINAL,
    // At the engine's noon scale: one asset against a backdrop is no scene to adapt the exposure to.
    features: { ...features, exposure: { ...features.exposure, isEnabled: false } },
    hemiStrength: 1,
    isSkyDrawn: false,
    // An asset viewer shows the asset as it is dressed: no toolbar there takes its textures off.
    isTextured: true,
    tonemapScale: 1,
  };
}
