import { formatCount } from "@xrf/format";
import {
  ESceneObjectProperties,
  ESceneSpawnKind,
  SceneBinaryToolDescription,
  SceneBinaryToolsDescription,
  SceneObjectDescription,
  SceneObjectId,
  SceneObjectsDescription,
  SceneObjectToolDescription,
  SceneProblem,
  SceneToolDescription,
  SceneToolKind,
} from "@xrf/ipc/types/xrf-app-levels";
import { assertExhaustive, Maybe, Nullable } from "@xrf/types";
import { ITreeNode } from "@xrf/ui/tree";

import { describeSceneBinaryTool, getSceneToolLabel } from "@/tools/level-editor/lib/tools/scene-tools";

/** What one row of the editor's Outliner stands for. */
export enum ELevelEditorOutlinerRow {
  /** A tool of the scene: an object tool holding its objects, or a binary tool counted. */
  TOOL = "tool",
  /** One object of an object tool. */
  OBJECT = "object",
}

/** One row of the editor's Outliner. */
export type TLevelEditorOutlinerRow =
  | {
      kind: ELevelEditorOutlinerRow.TOOL;
      /** The tool's name, as its part files carry it. */
      tool: string;
      /** Its object count or binary tool counts, beside the label; null before it is read. */
      caption: Nullable<string>;
      /** How many of the scene's problems are the tool's. */
      problems: number;
    }
  | { kind: ELevelEditorOutlinerRow.OBJECT; object: SceneObjectDescription };

/** What the Outliner is built from: the open scene's tools, what was read of them, and their problems. */
export interface ILevelEditorOutlinerSources {
  /** Every tool the scene holds part files of, in the order the backend lists them. */
  tools: ReadonlyArray<SceneToolDescription>;
  /** The object tools' objects, once read. */
  objects: Nullable<SceneObjectsDescription>;
  /** The binary tools' counts, once read. */
  binaryTools: Nullable<SceneBinaryToolsDescription>;
  /** Every problem found in the scene, each naming its tool. */
  problems: ReadonlyArray<SceneProblem>;
}

/** What the Outliner lists of the scene's objects. */
export interface ILevelEditorOutlinerFilter {
  /** What an object's name, reference or section must hold, case aside; empty keeps everything. */
  text: string;
}

/** Orders names as a reader does: `box_2` before `box_10`. */
const NAME_ORDER: Intl.Collator = new Intl.Collator(undefined, { numeric: true, sensitivity: "base" });

/**
 * The row id of a tool, by its name.
 */
export function toLevelEditorOutlinerToolId(tool: string): string {
  return `tool:${tool}`;
}

/**
 * The row id of a scene object.
 */
export function toLevelEditorOutlinerObjectId(id: SceneObjectId): string {
  return `object:${id.tool}:${id.index}`;
}

/**
 * The scene as a `tool -> object` tree: tools in the backend's order, each tool's objects by name.
 *
 * Every tool is listed, a binary tool as a leaf with its counts, then any tool named only by objects (a group's members
 * of a tool with no part) or problems. A filter keeps the objects it matches and only the tools holding one.
 */
export function toLevelEditorOutlinerTree(
  sources: ILevelEditorOutlinerSources,
  filter: ILevelEditorOutlinerFilter
): Array<ITreeNode<TLevelEditorOutlinerRow>> {
  const needle: string = filter.text.trim().toLowerCase();
  const objectTools: ReadonlyMap<string, SceneObjectToolDescription> = new Map(
    (sources.objects?.tools ?? []).map((it: SceneObjectToolDescription) => [it.tool, it])
  );
  const binaryTools: ReadonlyMap<string, SceneBinaryToolDescription> = new Map(
    (sources.binaryTools?.tools ?? []).map((it: SceneBinaryToolDescription) => [it.tool, it])
  );
  const problems: Map<string, number> = new Map();

  for (const problem of sources.problems) {
    problems.set(problem.tool, (problems.get(problem.tool) ?? 0) + 1);
  }

  // Keyed by name, which every source shares.
  const tools: Map<string, Nullable<SceneToolKind>> = new Map(
    sources.tools.map((it: SceneToolDescription) => [it.name, it.kind])
  );

  for (const tool of [...objectTools.values(), ...binaryTools.values()]) {
    if (!tools.has(tool.tool)) {
      tools.set(tool.tool, tool.tool);
    }
  }

  for (const tool of problems.keys()) {
    if (!tools.has(tool)) {
      tools.set(tool, null);
    }
  }

  return Array.from(tools).flatMap(
    ([tool, kind]: [string, Nullable<SceneToolKind>]): Array<ITreeNode<TLevelEditorOutlinerRow>> => {
      const objects: Maybe<SceneObjectToolDescription> = objectTools.get(tool);
      const binary: Maybe<SceneBinaryToolDescription> = binaryTools.get(tool);
      const kept: Array<SceneObjectDescription> = objects
        ? objects.objects.filter((object: SceneObjectDescription) => !needle || isMatched(object, needle))
        : [];

      if (needle && !kept.length) {
        return [];
      }

      return [
        {
          ...(kept.length ? { children: toObjectNodes(kept) } : {}),
          id: toLevelEditorOutlinerToolId(tool),
          label: getSceneToolLabel(kind, tool),
          payload: {
            caption: objects ? formatCount(kept.length) : binary ? describeSceneBinaryTool(binary) : null,
            kind: ELevelEditorOutlinerRow.TOOL,
            problems: problems.get(tool) ?? 0,
            tool,
          },
        },
      ];
    }
  );
}

/**
 * The ids of rows holding others, which a filter opens so its matches show.
 */
export function listLevelEditorOutlinerParentIds(
  nodes: ReadonlyArray<ITreeNode<TLevelEditorOutlinerRow>>
): Array<string> {
  return nodes.flatMap((node: ITreeNode<TLevelEditorOutlinerRow>) => (node.children?.length ? [node.id] : []));
}

/**
 * How many objects an Outliner tree lists.
 */
export function countLevelEditorOutlinerObjects(nodes: ReadonlyArray<ITreeNode<TLevelEditorOutlinerRow>>): number {
  return nodes.reduce((sum: number, node: ITreeNode<TLevelEditorOutlinerRow>) => sum + (node.children?.length ?? 0), 0);
}

/** Objects as rows, by name. */
function toObjectNodes(objects: ReadonlyArray<SceneObjectDescription>): Array<ITreeNode<TLevelEditorOutlinerRow>> {
  return [...objects]
    .sort((first: SceneObjectDescription, second: SceneObjectDescription) =>
      NAME_ORDER.compare(first.name, second.name)
    )
    .map((object: SceneObjectDescription): ITreeNode<TLevelEditorOutlinerRow> => ({
      id: toLevelEditorOutlinerObjectId(object.id),
      label: object.name,
      payload: { kind: ELevelEditorOutlinerRow.OBJECT, object },
    }));
}

/** Whether an object's name, reference or section holds `needle`, already lower-cased. */
function isMatched(object: SceneObjectDescription, needle: string): boolean {
  return [object.name, ...listSearchedProperties(object)].some((text: string) => text.toLowerCase().includes(needle));
}

/** What a filter reads of an object beyond its name: the library object or effect it places, the entity's section. */
function listSearchedProperties(object: SceneObjectDescription): Array<string> {
  const { properties } = object;

  switch (properties.kind) {
    case ESceneObjectProperties.SCENE_OBJECT:
    case ESceneObjectProperties.PARTICLE:
      return [properties.reference];

    case ESceneObjectProperties.SPAWN:
      return properties.spawn.kind === ESceneSpawnKind.ENTITY ? [properties.spawn.section] : [];

    case ESceneObjectProperties.SECTOR:
    case ESceneObjectProperties.PORTAL:
    case ESceneObjectProperties.LIGHT:
    case ESceneObjectProperties.GLOW:
    case ESceneObjectProperties.SHAPE:
    case ESceneObjectProperties.WAY:
    case ESceneObjectProperties.SOUND_SOURCE:
    case ESceneObjectProperties.SOUND_ENVIRONMENT:
    case ESceneObjectProperties.FOG_VOLUME:
    case ESceneObjectProperties.GROUP:
      return [];

    default:
      return assertExhaustive(properties);
  }
}
