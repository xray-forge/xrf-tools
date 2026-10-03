import { ERendererDebugView } from "@/core/render/lib/contract/renderer-debug-view";
import { IRendererSettings } from "@/core/render/lib/contract/renderer-settings";
import { IRenderSharedSettings } from "@/core/render/lib/settings/render-shared-settings";

/**
 * The renderer's settings for one asset against a backdrop, which the texture and visual viewers both draw.
 *
 * @param view - What the viewer decides: its backdrop and its toggles.
 * @param shared - What the application sets for every viewport.
 * @returns The renderer's settings.
 */
export function toAssetRendererSettings(
  view: Pick<IRendererSettings, "backdrop" | "isBumped" | "isLit" | "isWireframe">,
  shared: IRenderSharedSettings
): IRendererSettings {
  const { features } = shared;

  return {
    ...shared,
    ...view,
    debugView: ERendererDebugView.FINAL,
    // At the engine's noon scale: one asset against a backdrop is no scene to adapt the exposure to.
    features: { ...features, exposure: { ...features.exposure, isEnabled: false } },
    hemiStrength: 1,
    isSkyDrawn: false,
    isSkyHazed: false,
    // An asset viewer shows the asset as it is dressed: no toolbar there takes its textures off.
    isTextured: true,
    isWallmarkDrawn: true,
    tonemapScale: 1,
  };
}
