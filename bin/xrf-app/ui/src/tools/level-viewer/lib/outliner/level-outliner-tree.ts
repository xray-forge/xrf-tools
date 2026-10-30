import {
  ELevelObjectGroup,
  LevelObject,
  LevelObjectKind,
  LevelObjectKindEntry,
  LevelObjectsDescription,
} from "@xrf/ipc/types/xrf-app-levels";
import { Nullable } from "@xrf/types";
import { ITreeNode } from "@xrf/ui/tree";

import {
  ILevelObjectGroupEntry,
  LEVEL_OBJECT_GROUPS,
  listLevelObjectGroupKinds,
  toLevelObjectKey,
} from "@/core/level/lib/objects/level-object-kinds";

/** How many objects a group row holds, and how many of them a new game releases. */
interface ILevelOutlinerCount {
  count: number;
  released: number;
}

/** What one row of the Outliner stands for: a family of kinds, a kind, a section of a spawned kind, or one object. */
export type TLevelOutlinerRow =
  | ({ kind: "group"; entry: ILevelObjectGroupEntry; kinds: ReadonlyArray<LevelObjectKindEntry> } & ILevelOutlinerCount)
  | ({ kind: "kind"; entry: LevelObjectKindEntry } & ILevelOutlinerCount)
  | ({ kind: "section"; section: string } & ILevelOutlinerCount)
  | { kind: "object"; object: LevelObject };

/** What the Outliner lists of the level's objects. */
export interface ILevelOutlinerFilter {
  /** What a name, section or visual must hold, case aside; empty keeps everything. */
  text: string;
  /** The kinds kept; empty keeps every kind. */
  kinds: ReadonlySet<LevelObjectKind>;
}

/** Orders names as a reader does: `box_2` before `box_10`. */
const NAME_ORDER: Intl.Collator = new Intl.Collator(undefined, { numeric: true, sensitivity: "base" });

/**
 * @param kind - A kind.
 * @returns Its row's id.
 */
export function toLevelOutlinerKindId(kind: LevelObjectKind): string {
  return `kind:${kind}`;
}

/**
 * @param kind - A spawned kind.
 * @param section - A section its objects are of.
 * @returns Its row's id, which groups the kind's objects of the section.
 */
export function toLevelOutlinerSectionId(kind: LevelObjectKind, section: string): string {
  return `section:${kind}:${section}`;
}

/**
 * @param object - A level object.
 * @returns Its row's id.
 */
export function toLevelOutlinerObjectId(object: LevelObject): string {
  return `object:${toLevelObjectKey(object.id)}`;
}

/**
 * @param object - A level object.
 * @returns The rows above its own, from its group down: what opens to show it.
 */
export function listLevelOutlinerParentIds(object: LevelObject): Array<string> {
  const group: Nullable<ILevelObjectGroupEntry> =
    LEVEL_OBJECT_GROUPS.find((entry: ILevelObjectGroupEntry) =>
      listLevelObjectGroupKinds(entry.group).some((kind: LevelObjectKindEntry) => kind.kind === object.kind)
    ) ?? null;
  const ids: Array<string> = group ? [`group:${group.group}`, toLevelOutlinerKindId(object.kind)] : [];

  return object.spawn && group?.group === ELevelObjectGroup.SPAWNED
    ? [...ids, toLevelOutlinerSectionId(object.kind, object.spawn.section)]
    : ids;
}

/**
 * The level's objects as a tree, `Group -> kind -> object`, a spawned kind's objects by section between, each group
 * counting what it holds, only the objects a filter keeps, and a group left out where it holds none.
 *
 * @param description - The objects, and the visuals they are drawn as.
 * @param filter - What to keep.
 * @returns The groups and kinds in the kind table's order, sections and objects by name.
 */
export function toLevelOutlinerTree(
  description: LevelObjectsDescription,
  filter: ILevelOutlinerFilter
): Array<ITreeNode<TLevelOutlinerRow>> {
  const needle: string = filter.text.trim().toLowerCase();
  const matched: Array<LevelObject> = description.objects.filter(
    (object: LevelObject) =>
      (!filter.kinds.size || filter.kinds.has(object.kind)) &&
      (!needle ||
        [object.name, object.spawn?.section ?? "", toVisual(description, object)].some((text: string) =>
          text.toLowerCase().includes(needle)
        ))
  );

  return LEVEL_OBJECT_GROUPS.flatMap((group: ILevelObjectGroupEntry): Array<ITreeNode<TLevelOutlinerRow>> => {
    const kinds: Array<LevelObjectKindEntry> = listLevelObjectGroupKinds(group.group);
    const children: Array<ITreeNode<TLevelOutlinerRow>> = kinds.flatMap(
      (entry: LevelObjectKindEntry): Array<ITreeNode<TLevelOutlinerRow>> => {
        const objects: Array<LevelObject> = matched.filter((it: LevelObject) => it.kind === entry.kind);

        return objects.length
          ? [
              {
                children:
                  group.group === ELevelObjectGroup.SPAWNED
                    ? toSectionNodes(entry.kind, objects)
                    : toObjectNodes(objects),
                id: toLevelOutlinerKindId(entry.kind),
                label: entry.label,
                payload: { entry, kind: "kind", ...countObjects(objects) },
              },
            ]
          : [];
      }
    );
    const objects: Array<LevelObject> = matched.filter((it: LevelObject) =>
      kinds.some((entry: LevelObjectKindEntry) => entry.kind === it.kind)
    );

    return children.length
      ? [
          {
            children,
            id: `group:${group.group}`,
            label: group.label,
            payload: { entry: group, kind: "group", kinds, ...countObjects(objects) },
          },
        ]
      : [];
  });
}

/**
 * @param nodes - An Outliner tree.
 * @returns Every group's, kind's and section's id, which a filter opens so what it matched shows.
 */
export function listLevelOutlinerGroupIds(nodes: ReadonlyArray<ITreeNode<TLevelOutlinerRow>>): Array<string> {
  return nodes.flatMap((node: ITreeNode<TLevelOutlinerRow>): Array<string> =>
    node.payload?.kind === "object" ? [] : [node.id, ...listLevelOutlinerGroupIds(node.children ?? [])]
  );
}

/**
 * @param nodes - An Outliner tree.
 * @returns Its objects in the order the tree lists them, which a range of rows is chosen by.
 */
export function listLevelOutlinerObjects(nodes: ReadonlyArray<ITreeNode<TLevelOutlinerRow>>): Array<LevelObject> {
  return nodes.flatMap((node: ITreeNode<TLevelOutlinerRow>): Array<LevelObject> =>
    node.payload?.kind === "object" ? [node.payload.object] : listLevelOutlinerObjects(node.children ?? [])
  );
}

/** The visual an object is drawn as, empty for one drawn as nothing. */
function toVisual(description: LevelObjectsDescription, object: LevelObject): string {
  const visual: Nullable<number> = object.spawn?.visual ?? null;

  return visual === null ? "" : (description.visuals[visual] ?? "");
}

/** Objects as rows, by name. */
function toObjectNodes(objects: ReadonlyArray<LevelObject>): Array<ITreeNode<TLevelOutlinerRow>> {
  return objects
    .slice()
    .sort((first: LevelObject, second: LevelObject) => NAME_ORDER.compare(first.name, second.name))
    .map((object: LevelObject) => ({
      id: toLevelOutlinerObjectId(object),
      label: object.name,
      payload: { kind: "object" as const, object },
    }));
}

/** One spawned kind's objects by section, each section's objects by name. */
function toSectionNodes(
  kind: LevelObjectKind,
  objects: ReadonlyArray<LevelObject>
): Array<ITreeNode<TLevelOutlinerRow>> {
  const sections: Map<string, Array<LevelObject>> = new Map();

  for (const object of objects) {
    const section: string = object.spawn?.section ?? "";
    const group: Array<LevelObject> = sections.get(section) ?? [];

    group.push(object);
    sections.set(section, group);
  }

  return Array.from(sections.entries())
    .sort(([first], [second]) => NAME_ORDER.compare(first, second))
    .map(([section, members]: [string, Array<LevelObject>]) => ({
      children: toObjectNodes(members),
      id: toLevelOutlinerSectionId(kind, section),
      label: section,
      payload: { kind: "section" as const, section, ...countObjects(members) },
    }));
}

/** How many objects there are, and how many of them a new game releases. */
function countObjects(objects: ReadonlyArray<LevelObject>): ILevelOutlinerCount {
  return {
    count: objects.length,
    released: objects.filter((object: LevelObject) => Boolean(object.spawn?.release)).length,
  };
}
