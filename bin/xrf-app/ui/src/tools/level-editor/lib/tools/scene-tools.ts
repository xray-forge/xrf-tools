import { formatCount } from "@xrf/format";
import {
  ESceneBinaryToolDescription,
  ESceneToolKind,
  SceneBinaryToolDescription,
  SceneToolKind,
} from "@xrf/ipc/types/xrf-app-levels";
import { assertExhaustive, Nullable } from "@xrf/types";

/** What the editor's panels call each tool the SDK's Level Editor saves a part for. */
export const SCENE_TOOL_LABELS: Readonly<Record<ESceneToolKind, string>> = {
  [ESceneToolKind.GROUP]: "Groups",
  [ESceneToolKind.GLOW]: "Glows",
  [ESceneToolKind.SCENE_OBJECT]: "Scene objects",
  [ESceneToolKind.LIGHT]: "Lights",
  [ESceneToolKind.SHAPE]: "Shapes",
  [ESceneToolKind.SOUND_SOURCE]: "Sound sources",
  [ESceneToolKind.SPAWN]: "Spawn points",
  [ESceneToolKind.WAY]: "Ways",
  [ESceneToolKind.SECTOR]: "Sectors",
  [ESceneToolKind.PORTAL]: "Portals",
  [ESceneToolKind.SOUND_ENVIRONMENT]: "Sound environments",
  [ESceneToolKind.PARTICLE]: "Particles",
  [ESceneToolKind.DETAIL_OBJECT]: "Detail objects",
  [ESceneToolKind.AI_MAP]: "AI map",
  [ESceneToolKind.WALLMARK]: "Wallmarks",
  [ESceneToolKind.FOG_VOLUME]: "Fog volumes",
};

/**
 * @param kind - The editor's tool, none for one it does not save.
 * @param name - The tool's name, as its part files carry it.
 * @returns What the panels call it: its label, or its own name for a tool the editor does not save.
 */
export function getSceneToolLabel(kind: Nullable<SceneToolKind>, name: string): string {
  return kind === null ? name : SCENE_TOOL_LABELS[kind];
}

/**
 * @param tool - What one binary tool holds, counted.
 * @returns Its counts, as a row says them beside the tool's name.
 */
export function describeSceneBinaryTool(tool: SceneBinaryToolDescription): string {
  switch (tool.tool) {
    case ESceneBinaryToolDescription.AI_MAP:
      return `${formatCount(tool.nodes)} nodes`;

    case ESceneBinaryToolDescription.DETAIL_OBJECT:
      return `${formatCount(tool.objects)} objects · ${formatCount(tool.slots)} slots`;

    case ESceneBinaryToolDescription.WALLMARK:
      return `${formatCount(tool.marks)} marks · ${formatCount(tool.slots)} slots`;

    default:
      return assertExhaustive(tool);
  }
}
