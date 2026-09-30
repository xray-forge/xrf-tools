import { LevelSpawnCategory, LevelSpawnObject, LevelSpawnObjectsDescription } from "@/core/ipc/types/xrf-app";
import { ILevelSpawnCategoryEntry, LEVEL_SPAWN_CATEGORIES } from "@/core/level/lib/spawn/level-spawn-categories";
import { ITreeNode } from "@/core/ui/tree/tree-node";

/** What one row of the spawn tree stands for: a category, a section within it, or one object. */
export type TLevelSpawnTreeRow =
  | { kind: "category"; entry: ILevelSpawnCategoryEntry; count: number }
  | { kind: "section"; section: string; count: number }
  | { kind: "object"; object: LevelSpawnObject };

/** Orders names as a reader does: `box_2` before `box_10`. */
const NAME_ORDER: Intl.Collator = new Intl.Collator(undefined, { numeric: true, sensitivity: "base" });

/**
 * @param category - A category.
 * @returns Its row's id.
 */
export function toLevelSpawnCategoryId(category: LevelSpawnCategory): string {
  return `category:${category}`;
}

/**
 * @param index - An object, by its place among the level's spawned objects.
 * @returns Its row's id.
 */
export function toLevelSpawnObjectId(index: number): string {
  return `object:${index}`;
}

/**
 * The level's spawned objects as a tree, `Category -> section -> object`, each group counting what it holds, only the
 * objects a filter matches by name, section or visual kept, and a group left out where it holds none.
 *
 * @param description - The objects, and the visuals they stand as.
 * @param filter - What to match, case aside; empty keeps everything.
 * @returns The categories in the viewer's order, sections and objects by name.
 */
export function toLevelSpawnTree(
  description: LevelSpawnObjectsDescription,
  filter: string
): Array<ITreeNode<TLevelSpawnTreeRow>> {
  const needle: string = filter.trim().toLowerCase();
  const matched: Array<LevelSpawnObject> = needle
    ? description.objects.filter((object: LevelSpawnObject) =>
        [object.name, object.section, description.visuals[object.visual] ?? ""].some((text: string) =>
          text.toLowerCase().includes(needle)
        )
      )
    : description.objects;

  return LEVEL_SPAWN_CATEGORIES.flatMap((entry: ILevelSpawnCategoryEntry): Array<ITreeNode<TLevelSpawnTreeRow>> => {
    const objects: Array<LevelSpawnObject> = matched.filter((it: LevelSpawnObject) => it.category === entry.category);

    if (!objects.length) {
      return [];
    }

    return [
      {
        children: toSectionNodes(entry.category, objects),
        id: toLevelSpawnCategoryId(entry.category),
        label: entry.label,
        payload: { count: objects.length, entry, kind: "category" },
      },
    ];
  });
}

/**
 * @param nodes - A spawn tree.
 * @returns Every category's and section's id, which a filter opens so what it matched shows.
 */
export function listLevelSpawnGroupIds(nodes: ReadonlyArray<ITreeNode<TLevelSpawnTreeRow>>): Array<string> {
  return nodes.flatMap((category: ITreeNode<TLevelSpawnTreeRow>) => [
    category.id,
    ...(category.children ?? []).map((section: ITreeNode<TLevelSpawnTreeRow>) => section.id),
  ]);
}

/** One category's objects by section, each section's objects by name. */
function toSectionNodes(
  category: LevelSpawnCategory,
  objects: ReadonlyArray<LevelSpawnObject>
): Array<ITreeNode<TLevelSpawnTreeRow>> {
  const sections: Map<string, Array<LevelSpawnObject>> = new Map();

  for (const object of objects) {
    const group: Array<LevelSpawnObject> = sections.get(object.section) ?? [];

    group.push(object);
    sections.set(object.section, group);
  }

  return Array.from(sections.entries())
    .sort(([first], [second]) => NAME_ORDER.compare(first, second))
    .map(([section, members]: [string, Array<LevelSpawnObject>]) => ({
      children: members
        .slice()
        .sort((first: LevelSpawnObject, second: LevelSpawnObject) => NAME_ORDER.compare(first.name, second.name))
        .map((object: LevelSpawnObject) => ({
          id: toLevelSpawnObjectId(object.index),
          label: object.name,
          payload: { kind: "object" as const, object },
        })),
      id: `section:${category}:${section}`,
      label: section,
      payload: { count: members.length, kind: "section" as const, section },
    }));
}
