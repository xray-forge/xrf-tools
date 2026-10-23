// Auto-generated rust bindings. Do not edit it manually.

import { invoke as __TAURI_INVOKE } from "@/core/ipc/invoke";
import {
  CubeExportRequest,
  LevelConsoleDefaults,
  LevelEntry,
  LevelOpenRequest,
  LevelSpawnObjectDetails,
  LevelSpawnObjectsDescription,
  LevelWeatherCycle,
  LevelWeatherDescription,
  MapAnnotationsRequest,
  MapExportRequest,
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
  /** Copy a screenshot of the screenshots folder to where the page names, as Save as does. */
  copyScreenshot: (path: string, target: string) =>
    __TAURI_INVOKE<null>("plugin:levels|copy_screenshot", { path, target }),
  /**
   * Delete a screenshot of the screenshots folder for good, with what stands beside it: a screenshot's report, a level
   * map's annotations.
   */
  deleteScreenshot: (path: string) => __TAURI_INVOKE<null>("plugin:levels|delete_screenshot", { path }),
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
  /** Write a cubemap of the screenshots folder again as a cube DDS, six TGAs or six PNGs; answers the files written. */
  exportCube: (request: CubeExportRequest) => __TAURI_INVOKE<Array<string>>("plugin:levels|export_cube", { request }),
  /**
   * Write a level map of the screenshots folder again as a PNG, a TGA or a DDS, or its annotations alone as an SVG;
   * answers its pixels across and down.
   */
  exportMap: (request: MapExportRequest) => __TAURI_INVOKE<[number, number]>("plugin:levels|export_map", { request }),
  /**
   * The first free path for a screenshot named `name` in the screenshots folder, `-2`, `-3` and on added for one taken
   * within the same second.
   */
  findScreenshotPath: (name: string) => __TAURI_INVOKE<string>("plugin:levels|find_screenshot_path", { name }),
  /**
   * Restore the committed level descriptor without reading the level again; only where it starts is decided again,
   * where a commit let it go.
   */
  getLevel: () => __TAURI_INVOKE<SessionRestore<SelectedLevelDescription>>("plugin:levels|get_level"),
  /** The folder the level viewer's views are exported to and imported from by default, made where it is not yet. */
  getViewFolder: () => __TAURI_INVOKE<string>("plugin:levels|get_view_folder"),
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
  /** Read a level view's JSON file as text, for the page to check and import. */
  readView: (path: string) => __TAURI_INVOKE<string>("plugin:levels|read_view", { path }),
  /** Write a level view as its JSON text to the JSON file the page names. */
  saveView: (path: string, text: string) => __TAURI_INVOKE<null>("plugin:levels|save_view", { path, text }),
  /** Draw the open level's annotations over one of its level maps and write them beside it as an SVG; answers the SVG. */
  writeMapAnnotations: (sessionId: SessionId, request: MapAnnotationsRequest) =>
    __TAURI_INVOKE<SessionSnapshot<string>>("plugin:levels|write_map_annotations", { sessionId, request }),
  /** Write a screenshot's report beside it in the screenshots folder, as its JSON text; answers where it was written. */
  writeScreenshotReport: (path: string, text: string) =>
    __TAURI_INVOKE<string>("plugin:levels|write_screenshot_report", { path, text }),
};
