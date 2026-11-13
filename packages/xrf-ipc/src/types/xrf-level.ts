// Auto-generated rust bindings. Do not edit it manually.

import { Vector3d } from "#/types/xrf-math";

/** What a level's AI map held, beside what its graph check found. */
export type LevelAiGraphCensus = {
  nodes: number;
  /** Links naming a node of the map, each direction counted on its own. */
  links: number;
  /** Links naming the node they leave, which the engine's path search cannot take. */
  selfLinks: number;
  /** Links whose node does not link back across the same edge. */
  oneWayLinks: number;
  /** The nodes such a link reaches, as xrAI's `-verify` counts its "single linked nodes". */
  oneWayNodes: number;
  /** Parts of the map no link joins, a link counting either way. */
  components: number;
  largestComponent: number;
};

/** One local override of the level's weather, `CEnvModifier` (`xrEngine/Environment_misc.cpp`). */
export type LevelEnvModifier = {
  position: Vector3d;
  radius: number | null;
  power: number | null;
  farPlane: number | null;
  fogColor: Vector3d;
  fogDensity: number | null;
  ambient: Vector3d;
  skyColor: Vector3d;
  hemiColor: Vector3d;
  /**
   * Which values the engine actually mixes in. A file below version `0x0016` carries none and the engine uses all
   * of them, which is what `use_flags.one()` does before the read.
   */
  useFlags: number | null;
};
