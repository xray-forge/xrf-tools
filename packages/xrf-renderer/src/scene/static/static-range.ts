import type { StaticArena } from "#/scene/static/static-arena";

/**
 * Where one geometry's vertices and indices sit in a static arena: its indices as the geometry stores them, its
 * vertices from `vertexStart`, which a draw of it names as its base vertex.
 */
export interface IStaticRange {
  arena: StaticArena;
  vertexStart: number;
  vertexCount: number;
  indexStart: number;
  indexCount: number;
}
