import { SceneDrawDescription, SceneHelpersDescription } from "@xrf/ipc/types/xrf-app-levels";
import { RenderHelperOptions, RenderViewOptions } from "@xrf/ipc/types/xrf-renderer";
import { VisualBounds } from "@xrf/ipc/types/xrf-visual";
import { WorldToggles } from "@xrf/ipc/types/xrf-world";
import { IPoint3, toPoint3 } from "@xrf/math";
import { Nullable } from "@xrf/types";

import { ILevelSurfaceOptions } from "@/core/level/lib/surface/level-surface-options";
import { IRenderLighting, toNativeAssetLighting } from "@/core/render/lib/lighting/render-lighting";
import { toNativeAssetViewOptions, toNativeWorldToggles } from "@/core/render/lib/native/contract/native-view-options";
import { IRenderFeatureSettings } from "@/core/render/lib/settings/choice/render-feature-settings";
import { TRenderOutput } from "@/core/render/lib/settings/device/render-resolution";

/**
 * What a scene is lit by: a grey noon in place of a weather, since a scene carries no baked light and the editor shows
 * shape and placement rather than the final look.
 *
 * Bright enough to read the game's dark albedo, the sun high and off axis so walls and floors part by their shade, and
 * the ambient light whole so a room the sun does not reach still reads.
 */
export const DEFAULT_LEVEL_EDITOR_LIGHTING: IRenderLighting = {
  ambientColor: 0xffffff,
  ambientIntensity: 1,
  sunAzimuth: 143,
  sunColor: 0xffffff,
  sunElevation: 60,
  sunIntensity: 1,
};

/**
 * What of a scene's world plays: its objects, every one of which stands in the props group, and nothing else, since a
 * scene plays no weather, burns no campfire and has no sound.
 */
export const LEVEL_EDITOR_WORLD_TOGGLES: WorldToggles = toNativeWorldToggles(
  {
    isAmbientEffectPlayed: false,
    isCampfireLit: false,
    isRainy: false,
    isSpawnedItems: false,
    isSpawnedLamps: false,
    isSpawnedProps: true,
    isSpawnedReleased: false,
    isSpawnedWeapons: false,
    isThundering: false,
    isWindy: false,
  },
  [],
  { isAmbience: false, isHeard: false, isMusic: false, isWorld: false }
);

/** What a scene's view options are made of. */
export interface ILevelEditorViewOptionsInputs {
  /** How its surfaces are drawn: bumped, or as wires. */
  surfaces: Pick<ILevelSurfaceOptions, "isBumped" | "isWireframe">;
  /** What the tool sets every viewport's features to. */
  features: IRenderFeatureSettings;
  /** What lights the scene in place of a weather. */
  lighting: IRenderLighting;
  /** What the scene's frame is resolved at before the viewport. */
  output: TRenderOutput;
  /** Which of the scene's helpers are drawn, and how. */
  helpers: RenderHelperOptions;
}

/**
 * @param inputs - What the options are made of.
 * @returns What a native viewport draws a scene with: textured, under the editor's light, with no sky, fog or weather,
 *   its backdrop the level viewer's own, its helpers as switched.
 */
export function toLevelEditorViewOptions(inputs: ILevelEditorViewOptionsInputs): RenderViewOptions {
  const { surfaces, features, lighting, output, helpers } = inputs;
  const view: RenderViewOptions = toNativeAssetViewOptions(
    { isBumped: surfaces.isBumped, isLit: true, isWireframe: surfaces.isWireframe },
    features,
    output
  );

  return { ...view, asset: { ...view.asset, lighting: toNativeAssetLighting(lighting) }, helpers };
}

/**
 * @param drawing - What the open scene's picture needs, or null before it is read.
 * @param helpers - What the page needs of the open scene's helpers, or null before it is read.
 * @returns What a camera frames the scene by: the extent of its drawn objects where it draws any, else that of its
 *   helpers once they are read; null while that is not known yet and for a scene drawing neither.
 */
export function toLevelEditorSceneBounds(
  drawing: Nullable<SceneDrawDescription>,
  helpers: Nullable<SceneHelpersDescription>
): Nullable<VisualBounds> {
  if (!drawing) {
    return null;
  }

  return drawing.bounds ?? helpers?.bounds ?? null;
}

/**
 * @param bounds - The extent of a scene's drawn objects, in renderer space, the sphere about their box stated.
 * @returns The sphere a camera frames them by; one stating no radius frames its centre alone.
 */
export function toLevelEditorSceneSphere(bounds: VisualBounds): { center: IPoint3; radius: number } {
  return { center: toPoint3(bounds.boundingSphere.center), radius: bounds.boundingSphere.radius ?? 0 };
}
