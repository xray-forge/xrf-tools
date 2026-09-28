import { Nullable } from "@xrf/types";
import { BufferGeometry, StorageBufferNode } from "three/webgpu";

import { IClusterAttribute } from "#/geometry/cluster-attribute";

/**
 * Where a clustered draw reads its vertices: the entry its instance is, the cluster it names, words, a vertex its
 * attributes one after another, and the indices naming them, each through the one node every shader over them reads.
 */
export interface IClusterSource {
  /** Every view's kept clusters, each a cluster and its place, which an instance of a clustered draw is one of. */
  readonly entryNode: StorageBufferNode<"uvec2">;
  /** Every cluster's first index and base vertex in its arena, its triangles, and its slot. */
  readonly rangeNode: StorageBufferNode<"uvec4">;
  readonly wordNode: StorageBufferNode<"uint">;
  readonly indexNode: StorageBufferNode<"uint">;
  /** Words a vertex takes. */
  readonly stride: number;
  /** Each attribute's place among a vertex's words. */
  readonly layout: ReadonlyArray<IClusterAttribute>;
}

/**
 * @param geometry - A geometry a shader is built for, or none.
 * @returns Where it draws its clusters' vertices from, for a clustered draw's geometry; null for every other, which
 *   reads its vertices from its own attributes.
 */
export function toClusterSource(geometry: Nullable<BufferGeometry> | undefined): Nullable<IClusterSource> {
  return (geometry?.userData?.clusters as Nullable<IClusterSource> | undefined) ?? null;
}
