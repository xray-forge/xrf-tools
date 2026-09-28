import { TypedArray } from "three/webgpu";

import { EClusterWordFormat } from "#/geometry/cluster-word-format";

/**
 * One vertex attribute as a clustered draw's source stores it, and where it sits among a vertex's words.
 */
export interface IClusterAttribute {
  name: string;
  type: new (length: number) => TypedArray;
  itemSize: number;
  isNormalized: boolean;
  format: EClusterWordFormat;
  /** Its first word, from the vertex's first. */
  offset: number;
  /** Words it takes. */
  words: number;
}
