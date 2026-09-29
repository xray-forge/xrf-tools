// Auto-generated rust bindings. Do not edit it manually.

import { invoke as __TAURI_INVOKE } from "@/core/ipc/invoke";
import {
  LevelDetailsDescription,
  LevelEntry,
  LevelLightsDescription,
  LevelOpenRequest,
  LevelSpawnModelsDescription,
  LevelWeatherCycle,
  LevelWeatherDescription,
  SelectedLevelDescription,
  SessionId,
  SessionRestore,
  SessionSnapshot,
} from "@/core/ipc/types/xrf-app";
import { EWeatherCycleKind } from "@/core/ipc/types/xrf-environment";
import { XrayRoots } from "@/core/ipc/types/xrf-vfs";
import { SectorDescription } from "@/core/ipc/types/xrf-visual";

/** Commands */
export const levelsCommands = {
  /** Release only the openings owned by the departing viewer. */
  closeLevel: (sessionIds: Array<SessionId>) => __TAURI_INVOKE<null>("plugin:levels|close_level", { sessionIds }),
  /** Restore the committed level descriptor without reading the level again. */
  getLevel: () => __TAURI_INVOKE<SessionRestore<SelectedLevelDescription>>("plugin:levels|get_level"),
  /** Every compiled level the mounted roots hold, loose or archived alike. */
  listLevels: (roots: XrayRoots) => __TAURI_INVOKE<Array<LevelEntry>>("plugin:levels|list_levels", { roots }),
  /** Pack the open level's grass and describe it, or answer nothing for a level with no detail library. */
  openDetails: (sessionId: SessionId, detailsId: SessionId) =>
    __TAURI_INVOKE<SessionSnapshot<LevelDetailsDescription | null>>("plugin:levels|open_details", {
      sessionId,
      detailsId,
    }),
  /** Select a compiled level and report what it is built out of, without reading any of its geometry. */
  openLevel: (sessionId: SessionId, request: LevelOpenRequest) =>
    __TAURI_INVOKE<SessionSnapshot<SelectedLevelDescription>>("plugin:levels|open_level", { sessionId, request }),
  /** Collect the open level's lights: the lamps the game spawns on it, and its own. */
  openLights: (sessionId: SessionId) =>
    __TAURI_INVOKE<SessionSnapshot<LevelLightsDescription>>("plugin:levels|open_lights", { sessionId }),
  /** Pack one sector of the open level and report what it became. */
  openSector: (sessionId: SessionId, sectorId: SessionId, sector: number) =>
    __TAURI_INVOKE<SessionSnapshot<SectorDescription>>("plugin:levels|open_sector", { sessionId, sectorId, sector }),
  /** Describe the models the open level's spawned objects are drawn as, and where each object stands. */
  openSpawnModels: (sessionId: SessionId) =>
    __TAURI_INVOKE<SessionSnapshot<LevelSpawnModelsDescription>>("plugin:levels|open_spawn_models", { sessionId }),
  /**
   * Read any cycle or effect of the game as the open level's engine loads it, for a viewer playing one the level does
   * not offer itself.
   */
  readLevelCycle: (sessionId: SessionId, kind: EWeatherCycleKind, name: string) =>
    __TAURI_INVOKE<SessionSnapshot<LevelWeatherCycle>>("plugin:levels|read_level_cycle", { sessionId, kind, name }),
  /**
   * Read the open level's weather as its engine loads it: the cycles it plays, the effects, what they strike with,
   * the sun table and the level's own overrides.
   */
  readLevelWeather: (sessionId: SessionId) =>
    __TAURI_INVOKE<SessionSnapshot<LevelWeatherDescription>>("plugin:levels|read_level_weather", { sessionId }),
};
