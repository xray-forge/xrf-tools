import {
  ESceneToolKind,
  SceneDrawDescription,
  SceneHelpersDescription,
  SceneObjectId,
} from "@xrf/ipc/types/xrf-app-levels";
import { Sphere } from "@xrf/ipc/types/xrf-math";
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
import {
  ILevelEditorHelperOptions,
  isLevelEditorHelperOn,
} from "@/tools/level-editor/lib/helpers/level-editor-helper-options";
import { toSceneRenderModelIds } from "@/tools/level-editor/lib/render/scene-render-id";

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

/** What a scene's world toggles are made of. */
export interface ILevelEditorWorldTogglesInputs {
  /** The scene objects hidden one by one, of any tool. */
  hidden: ReadonlyArray<SceneObjectId>;
  /**
   * Which helpers the Show action has switched on: the spawn points' row, under the master switch, switches their
   * models with their helpers.
   */
  helpers: ILevelEditorHelperOptions;
  /** Whether the Show action draws the surfaces the game never draws. */
  isCollisionShown: boolean;
  /** Whether the Show action draws the surfaces dressed in the SDK's marker textures. */
  isEditorOnlyShown: boolean;
}

/**
 * @param inputs - What the toggles are made of.
 * @returns What of a scene's world plays: its scene objects, which stand in the props group; their surfaces the game
 *   never draws, which stand in the weapons group of their own, and those dressed in the SDK's marker textures, in the
 *   lamps group of their own, each shown as asked; and its spawn points' models, which stand in the items group of
 *   their own, shown as the spawn points' helpers are; less the objects hidden, of any tool, named by the render ids
 *   of every model they stand as, ascending; and nothing else, since a scene plays no weather, burns no campfire and
 *   has no sound.
 */
export function toLevelEditorWorldToggles(inputs: ILevelEditorWorldTogglesInputs): WorldToggles {
  const { hidden, helpers, isCollisionShown, isEditorOnlyShown } = inputs;

  return toNativeWorldToggles(
    {
      isAmbientEffectPlayed: false,
      isCampfireLit: false,
      isRainy: false,
      isSpawnedItems: helpers.isShown && isLevelEditorHelperOn(helpers, ESceneToolKind.SPAWN),
      isSpawnedLamps: isEditorOnlyShown,
      isSpawnedProps: true,
      isSpawnedReleased: false,
      isSpawnedWeapons: isCollisionShown,
      isThundering: false,
      isWindy: false,
    },
    toSceneRenderModelIds(hidden),
    { isAmbience: false, isHeard: false, isMusic: false, isWorld: false }
  );
}

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
 * @param sphere - What a camera frames, in renderer space: a scene's extent's sphere, or one object's.
 * @returns The sphere as a camera frames it; one stating no radius frames its centre alone.
 */
export function toLevelEditorSceneSphere(sphere: Sphere): { center: IPoint3; radius: number } {
  return { center: toPoint3(sphere.center), radius: sphere.radius ?? 0 };
}
