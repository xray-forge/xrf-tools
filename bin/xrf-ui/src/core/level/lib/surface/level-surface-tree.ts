import { ILevelSurfaceSummary } from "@/core/level/lib/surface/level-surface-summary";
import { ITreeNode } from "@/core/ui/tree/tree-node";

/** What one row of the surfaces tree stands for: a shader, or one entry of the table naming it. */
export type TLevelSurfaceTreeRow =
  { kind: "shader"; shader: string; count: number } | { kind: "entry"; summary: ILevelSurfaceSummary };

/** Orders shaders as a reader does: `lod_2` before `lod_10`. */
const NAME_ORDER: Intl.Collator = new Intl.Collator(undefined, { numeric: true, sensitivity: "base" });

/**
 * @param shaderId - An entry of the shader table.
 * @returns Its row's id.
 */
export function toLevelSurfaceEntryId(shaderId: number): string {
  return `entry:${shaderId}`;
}

/**
 * A level's shader table as a tree, `shader -> entry`: a level names one entry per shader and texture set, so the
 * shader groups what only the texture tells apart. Only the entries a filter matches by shader, texture or id are kept.
 *
 * @param summaries - The entries naming a shader.
 * @param filter - What to match, case aside; empty keeps everything.
 * @returns Shaders by name, each one's entries by id.
 */
export function toLevelSurfaceTree(
  summaries: ReadonlyArray<ILevelSurfaceSummary>,
  filter: string
): Array<ITreeNode<TLevelSurfaceTreeRow>> {
  const needle: string = filter.trim().toLowerCase();
  const shaders: Map<string, Array<ILevelSurfaceSummary>> = new Map();

  for (const summary of summaries) {
    if (needle && !isMatched(summary, needle)) {
      continue;
    }

    const group: Array<ILevelSurfaceSummary> = shaders.get(summary.shader) ?? [];

    group.push(summary);
    shaders.set(summary.shader, group);
  }

  return Array.from(shaders.entries())
    .sort(([first], [second]) => NAME_ORDER.compare(first, second))
    .map(([shader, entries]: [string, Array<ILevelSurfaceSummary>]) => ({
      children: entries.map((summary: ILevelSurfaceSummary) => ({
        id: toLevelSurfaceEntryId(summary.shaderId),
        label: `${summary.shaderId} · ${summary.textures[0] ?? "no texture"}`,
        payload: { kind: "entry" as const, summary },
      })),
      id: `shader:${shader}`,
      label: shader,
      payload: { count: entries.length, kind: "shader" as const, shader },
    }));
}

/**
 * @param nodes - A surfaces tree.
 * @returns Every shader's id, which a filter opens so what it matched shows.
 */
export function listLevelSurfaceGroupIds(nodes: ReadonlyArray<ITreeNode<TLevelSurfaceTreeRow>>): Array<string> {
  return nodes.map((node: ITreeNode<TLevelSurfaceTreeRow>) => node.id);
}

/** Whether an entry names what is sought: its shader, a texture, or its id. */
function isMatched(summary: ILevelSurfaceSummary, needle: string): boolean {
  return (
    summary.shader.toLowerCase().includes(needle) ||
    summary.textures.some((texture: string) => texture.toLowerCase().includes(needle)) ||
    String(summary.shaderId) === needle
  );
}
