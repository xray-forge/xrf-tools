// Auto-generated rust bindings. Do not edit it manually.

import { Vector3d } from "@/core/ipc/types/xrf-math";

/** One local override of the level's weather, `CEnvModifier` (`xrEngine/Environment_misc.cpp`). */
export type EnvModifier = {
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
