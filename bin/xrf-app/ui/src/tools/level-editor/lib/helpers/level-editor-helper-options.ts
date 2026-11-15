import { SCENE_HELPER_TOOLS, SceneHelperToolEntry, SceneObjectId, SceneToolKind } from "@xrf/ipc/types/xrf-app-levels";
import { RenderHelperOptions } from "@xrf/ipc/types/xrf-renderer";
import { isRecord, Nullable } from "@xrf/types";

import { NO_NATIVE_HELPERS } from "@/core/render/lib/native/contract/native-view-options";
import { toSceneHelperId, toSceneHelperIds } from "@/tools/level-editor/lib/helpers/scene-helper-id";

/** Which helpers the editor draws over what a scene holds with no geometry: a preference, kept over runs. */
export interface ILevelEditorHelperOptions {
  /** Draws the helpers at all, each tool keeping its choice either way. */
  isShown: boolean;
  /** Each helped tool's helpers as switched, by tool; a tool not named draws as the helper table says by default. */
  tools: Partial<Record<SceneToolKind, boolean>>;
}

/** Helped tools whose helpers are switched together, and whether they are drawn from now on. */
export interface ILevelEditorHelperToolsChange {
  tools: ReadonlyArray<SceneToolKind>;
  isOn: boolean;
}

/** Why a tool's helper switch cannot be turned while helpers are off. */
export const LEVEL_EDITOR_HELPERS_OFF_REASON: string = "Helpers are off: turn them on above";

/**
 * Helpers drawn, each tool as the helper table says: a scene holding nothing but spawn points, ways and the like opens
 * showing them.
 */
export const DEFAULT_LEVEL_EDITOR_HELPER_OPTIONS: ILevelEditorHelperOptions = { isShown: true, tools: {} };

/**
 * @param tool - A scene tool.
 * @returns Its row of the helper table, or null for a tool drawing no helper.
 */
export function getSceneHelperTool(tool: SceneToolKind): Nullable<SceneHelperToolEntry> {
  return SCENE_HELPER_TOOLS.find((entry: SceneHelperToolEntry) => entry.tool === tool) ?? null;
}

/**
 * @param options - Which helpers are drawn.
 * @param tool - A helped tool.
 * @returns Whether its helpers are switched on, the master switch aside; never for a tool drawing none.
 */
export function isLevelEditorHelperOn(options: ILevelEditorHelperOptions, tool: SceneToolKind): boolean {
  return options.tools[tool] ?? getSceneHelperTool(tool)?.isHelpedByDefault ?? false;
}

/**
 * @param stored - The parsed stored options, `null` where none were stored.
 * @returns What was stored, a switch it does not hold or a tool the table no longer helps left to its default.
 */
export function toLevelEditorHelperOptions(stored: unknown): ILevelEditorHelperOptions {
  if (!isRecord(stored)) {
    return DEFAULT_LEVEL_EDITOR_HELPER_OPTIONS;
  }

  const tools: Record<string, unknown> = isRecord(stored.tools) ? stored.tools : {};

  return {
    isShown: typeof stored.isShown === "boolean" ? stored.isShown : DEFAULT_LEVEL_EDITOR_HELPER_OPTIONS.isShown,
    tools: Object.fromEntries(
      SCENE_HELPER_TOOLS.filter((entry: SceneHelperToolEntry) => typeof tools[entry.tool] === "boolean").map(
        (entry: SceneHelperToolEntry) => [entry.tool, tools[entry.tool]]
      )
    ),
  };
}

/** The scene objects the editor singles out for their helpers. */
export interface ILevelEditorHelperTargets {
  /** Selected, which draws lifted and never fades; null for none. */
  selected: Nullable<SceneObjectId>;
  /** Hidden, which draw nothing. */
  hidden: ReadonlyArray<SceneObjectId>;
  /** Under the pointer, which draws lifted; null for none. */
  hovered: Nullable<SceneObjectId>;
}

/** Nothing singled out. */
export const NO_LEVEL_EDITOR_HELPER_TARGETS: ILevelEditorHelperTargets = { hidden: [], hovered: null, selected: null };

/**
 * @param options - Which helpers the editor draws.
 * @param targets - The scene objects singled out, each named to the renderer by its helper's id.
 * @returns What the renderer draws of a scene's helpers: each tool switched by its row's `kind`, as the backend numbers
 *   the helpers it draws, how they show left as every view's default, and the targets singled out.
 */
export function toLevelEditorHelperRenderOptions(
  options: ILevelEditorHelperOptions,
  targets: ILevelEditorHelperTargets = NO_LEVEL_EDITOR_HELPER_TARGETS
): RenderHelperOptions {
  const { selected, hidden, hovered } = targets;

  return {
    ...NO_NATIVE_HELPERS,
    hidden: toSceneHelperIds(hidden),
    hovered: hovered ? toSceneHelperId(hovered) : null,
    isShown: options.isShown,
    kinds: SCENE_HELPER_TOOLS.reduce(
      (bits: number, entry: SceneHelperToolEntry) =>
        isLevelEditorHelperOn(options, entry.tool) ? (bits | (1 << entry.kind)) >>> 0 : bits,
      0
    ),
    selected: selected ? toSceneHelperIds([selected]) : [],
  };
}
