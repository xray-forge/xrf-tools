import { SCENE_HELPER_TOOLS, SceneHelperToolEntry, SceneObjectId, SceneToolKind } from "@xrf/ipc/types/xrf-app-levels";
import { RenderHelperOptions } from "@xrf/ipc/types/xrf-renderer";
import { isRecord, Nullable } from "@xrf/types";

import { NO_NATIVE_HELPERS } from "@/core/render/lib/native/contract/native-view-options";
import { toSceneRenderId, toSceneRenderIds } from "@/tools/level-editor/lib/render/scene-render-id";

/** Which helpers the editor draws for objects with no geometry; a preference kept across runs. */
export interface ILevelEditorHelperOptions {
  /** Master switch; each tool keeps its own choice either way. */
  isShown: boolean;
  /** Each helped tool's switch; a tool not named follows the helper table's default. */
  tools: Partial<Record<SceneToolKind, boolean>>;
}

/** A switch of one helped tool's helpers. */
export interface ILevelEditorHelperToolChange {
  tool: SceneToolKind;
  isOn: boolean;
}

/** Why a tool's helper switch cannot be turned while helpers are off. */
export const LEVEL_EDITOR_HELPERS_OFF_REASON: string = "Helpers are off: turn them on above";

/**
 * Helpers on, each tool as the helper table says, so a scene of only spawn points, ways and the like opens showing them.
 */
export const DEFAULT_LEVEL_EDITOR_HELPER_OPTIONS: ILevelEditorHelperOptions = { isShown: true, tools: {} };

/**
 * A tool's row of the helper table, or null for a tool drawing no helper.
 */
export function getSceneHelperTool(tool: SceneToolKind): Nullable<SceneHelperToolEntry> {
  return SCENE_HELPER_TOOLS.find((entry: SceneHelperToolEntry) => entry.tool === tool) ?? null;
}

/**
 * Whether a tool's helpers are switched on, ignoring the master switch; false for a tool drawing none.
 */
export function isLevelEditorHelperOn(options: ILevelEditorHelperOptions, tool: SceneToolKind): boolean {
  return options.tools[tool] ?? getSceneHelperTool(tool)?.isHelpedByDefault ?? false;
}

/**
 * Stored options validated, a missing switch or a tool the table no longer helps left to its default.
 *
 * @param stored - The parsed stored options, `null` where none were stored.
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
 * The renderer's helper options: each switched tool as a bit of its row's `kind`, the targets by render id, and the
 * look left at every view's default.
 */
export function toLevelEditorHelperRenderOptions(
  options: ILevelEditorHelperOptions,
  targets: ILevelEditorHelperTargets = NO_LEVEL_EDITOR_HELPER_TARGETS
): RenderHelperOptions {
  const { selected, hidden, hovered } = targets;

  return {
    ...NO_NATIVE_HELPERS,
    hidden: toSceneRenderIds(hidden),
    hovered: hovered ? toSceneRenderId(hovered) : null,
    isShown: options.isShown,
    kinds: SCENE_HELPER_TOOLS.reduce(
      (bits: number, entry: SceneHelperToolEntry) =>
        isLevelEditorHelperOn(options, entry.tool) ? (bits | (1 << entry.kind)) >>> 0 : bits,
      0
    ),
    selected: selected ? toSceneRenderIds([selected]) : [],
  };
}
