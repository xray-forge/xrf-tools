// Auto-generated rust bindings. Do not edit it manually.

import { invoke as __TAURI_INVOKE } from "@/core/ipc/invoke";
import {
  LevelConsoleDefaults,
  LevelEntry,
  LevelOpenRequest,
  LevelSpawnObjectDetails,
  LevelSpawnObjectsDescription,
  LevelWeatherCycle,
  LevelWeatherDescription,
  SelectedLevelDescription,
  SessionId,
  SessionRestore,
  SessionSnapshot,
} from "@/core/ipc/types/xrf-app";
import { WeatherCycleId } from "@/core/ipc/types/xrf-environment";
import { XrayRoots } from "@/core/ipc/types/xrf-vfs";

/** Commands */
export const levelsCommands = {
  /** Release only the openings owned by the departing viewer. */
  closeLevel: (sessionIds: Array<SessionId>) => __TAURI_INVOKE<null>("plugin:levels|close_level", { sessionIds }),
  /**
   * Describe what the open level's game runs its console with of how levels are lit and exposed: its `user.ltx` over
   * its shipped defaults.
   */
  describeConsoleDefaults: (sessionId: SessionId) =>
    __TAURI_INVOKE<SessionSnapshot<LevelConsoleDefaults>>("plugin:levels|describe_console_defaults", { sessionId }),
  /** Describe one of the open level's spawned objects, by its place among them, as open_spawn_objects numbered it. */
  describeSpawnObject: (sessionId: SessionId, index: number) =>
    __TAURI_INVOKE<SessionSnapshot<LevelSpawnObjectDetails>>("plugin:levels|describe_spawn_object", {
      sessionId,
      index,
    }),
  /** Restore the committed level descriptor without reading the level again. */
  getLevel: () => __TAURI_INVOKE<SessionRestore<SelectedLevelDescription>>("plugin:levels|get_level"),
  /** Every compiled level the mounted roots hold, loose or archived alike. */
  listLevels: (roots: XrayRoots) => __TAURI_INVOKE<Array<LevelEntry>>("plugin:levels|list_levels", { roots }),
  /** Select a compiled level and report what it is built out of, without reading any of its geometry. */
  openLevel: (sessionId: SessionId, request: LevelOpenRequest) =>
    __TAURI_INVOKE<SessionSnapshot<SelectedLevelDescription>>("plugin:levels|open_level", { sessionId, request }),
  /**
   * Describe the open level's spawned objects the viewer draws, and the visuals they stand as, reading no visual; an
   * error where the spawn cannot be read.
   */
  openSpawnObjects: (sessionId: SessionId) =>
    __TAURI_INVOKE<SessionSnapshot<LevelSpawnObjectsDescription>>("plugin:levels|open_spawn_objects", { sessionId }),
  /**
   * Read any cycle or effect of the game as the open level's engine loads it, for a viewer playing one the level does
   * not offer itself.
   */
  readLevelCycle: (sessionId: SessionId, cycle: WeatherCycleId) =>
    __TAURI_INVOKE<SessionSnapshot<LevelWeatherCycle>>("plugin:levels|read_level_cycle", { sessionId, cycle }),
  /**
   * Read the open level's weather as its engine loads it: the cycles it plays, the effects, what they strike with,
   * the sun table and the level's own overrides.
   */
  readLevelWeather: (sessionId: SessionId) =>
    __TAURI_INVOKE<SessionSnapshot<LevelWeatherDescription>>("plugin:levels|read_level_weather", { sessionId }),
};
