import { ComputeNode } from "three/webgpu";

import { GrassRingUniforms } from "#/uniforms/grass-ring-uniforms";

/** The compute passes that plant a frame's grass, in the order a frame runs them, and how far their ring reaches. */
export interface IGrassPlanting {
  /** Fitted before each frame's to the planting's reach and what the build holds. */
  ring: GrassRingUniforms;
  /** Zeroes the stale cells each band of the ring counts. */
  clearSchedule: ComputeNode;
  /** Counts every stale cell of the ring into its band. */
  rank: ComputeNode;
  /** Finds the band the frame's planting budget runs out in, the nearest first. */
  select: ComputeNode;
  /** Plants the stale cells the schedule admits: `cache_Decompress`. */
  decompress: ComputeNode;
  /** Zeroes the frame's counts. */
  clear: ComputeNode;
  /** Culls what every current cell holds and appends what it keeps: `UpdateVisibleM`. */
  cull: ComputeNode;
  /** Lays each model's items out after the last's, and writes its draw. */
  arrange: ComputeNode;
  /** Sorts the items into each model's range. */
  scatter: ComputeNode;
}

/**
 * @param planting - The passes.
 * @returns Them in the order a frame dispatches them.
 */
export function listGrassPlantingPasses(planting: IGrassPlanting): Array<ComputeNode> {
  const { clearSchedule, rank, select, decompress, clear, cull, arrange, scatter } = planting;

  return [clearSchedule, rank, select, decompress, clear, cull, arrange, scatter];
}
