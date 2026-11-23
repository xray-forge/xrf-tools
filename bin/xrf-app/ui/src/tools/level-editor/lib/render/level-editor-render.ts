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
 * A grey noon lighting the scene in place of a weather: a scene carries no baked light, and the editor shows shape and
 * placement rather than the final look.
 *
 * Bright for the game's dark albedo, the sun high and off axis so walls and floors shade apart, and full ambient so
 * unlit rooms still read.
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
  /** Which helpers the Show action has switched on; the spawn points' switch also switches their models. */
  helpers: ILevelEditorHelperOptions;
  /** Whether the Show action draws the surfaces the game never draws. */
  isCollisionShown: boolean;
  /** Whether the Show action draws the surfaces dressed in the SDK's marker textures. */
  isEditorOnlyShown: boolean;
}

/**
 * The world toggles a scene plays with, less its hidden objects; no weather, campfires or sound.
 *
 * A scene reuses the level's groups: scene objects stand in props, collision surfaces in weapons, marker-textured
 * surfaces in lamps, and spawn point models in items, shown as the spawn points' helpers are.
 */
export function toLevelEditorWorldToggles(inputs: ILevelEditorWorldTogglesInputs): WorldToggles {
  const { hidden, helpers, isCollisionShown, isEditorOnlyShown } = inputs;

  return toNativeWorldToggles(
    {
      isAmbientEffectPlayed: false,
      isCampfireLit: false,
      isRainy: false,
      // A scene marks no artefact detector-hidden; its spawn point models stand with the items.
      isSpawnedDetectorHidden: true,
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
  surfaces: Pick<ILevelSurfaceOptions, "isBumped" | "isWireframe">;
  /** What the tool sets every viewport's features to. */
  features: IRenderFeatureSettings;
  /** What lights the scene in place of a weather. */
  lighting: IRenderLighting;
  /** The resolution the frame renders at before the viewport. */
  output: TRenderOutput;
  helpers: RenderHelperOptions;
}

/**
 * A native viewport's options for a scene: textured, under the editor's light, with no sky, fog or weather, and the
 * level viewer's backdrop.
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
 * The extent a camera frames the scene by: its drawn objects', else its helpers'.
 *
 * @returns The bounds, or null while the drawing is unread or for a scene drawing neither.
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
 * A renderer-space sphere as a camera frames it; one without a radius frames its centre alone.
 */
export function toLevelEditorSceneSphere(sphere: Sphere): { center: IPoint3; radius: number } {
  return { center: toPoint3(sphere.center), radius: sphere.radius ?? 0 };
}
