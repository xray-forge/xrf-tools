// Auto-generated rust bindings. Do not edit it manually.

import { Channel } from "@tauri-apps/api/core";

import { invoke as __TAURI_INVOKE } from "@/core/ipc/invoke";
import {
  CaptureOutcome,
  CubeCaptureRequest,
  CubeExportRequest,
  LevelConsoleDefaults,
  LevelEntry,
  LevelMapDescription,
  LevelOpenRequest,
  LevelSpawnObjectDetails,
  LevelSpawnObjectsDescription,
  LevelStart,
  LevelTextureReference,
  LevelWeatherCycle,
  LevelWeatherDescription,
  MapAnnotationsRequest,
  MapCaptureRequest,
  MapExportRequest,
  SelectedLevelDescription,
  SessionId,
  SessionRestore,
  SessionSnapshot,
} from "@/core/ipc/types/xrf-app";
import { WeatherCycleId } from "@/core/ipc/types/xrf-environment";
import { JobProgress } from "@/core/ipc/types/xrf-job";
import { XraySurfaceDescriptor } from "@/core/ipc/types/xrf-material";
import { RenderViewportId } from "@/core/ipc/types/xrf-renderer";
import { XrayRoots } from "@/core/ipc/types/xrf-vfs";

/** Commands */
export const levelsCommands = {
  /** Capture a cubemap around a viewport's camera face by face and write it to a PNG file as a horizontal cross. */
  captureCube: (request: CubeCaptureRequest, jobId: string, progress: Channel<JobProgress>) =>
    __TAURI_INVOKE<CaptureOutcome>("plugin:levels|capture_cube", { request, jobId, progress }),
  /** Capture a level map in a viewport tile by tile and write it to a PNG file with its coverage in alpha. */
  captureMap: (request: MapCaptureRequest, jobId: string, progress: Channel<JobProgress>) =>
    __TAURI_INVOKE<CaptureOutcome>("plugin:levels|capture_map", { request, jobId, progress }),
  /** Close the levels `session_ids` name, each once, unbinding it from its scope. */
  closeLevel: (sessionIds: Array<SessionId>) => __TAURI_INVOKE<void>("plugin:levels|close_level", { sessionIds }),
  /**
   * Copy a screenshot of the screenshots folder to where the page names, as Save as does; read whole first, so a failed
   * copy leaves no partial file and keeps any file it would have replaced.
   */
  copyScreenshot: (path: string, target: string) =>
    __TAURI_INVOKE<null>("plugin:levels|copy_screenshot", { path, target }),
  /**
   * Delete a screenshot of the screenshots folder for good, with what stands beside it: a screenshot's report, a level
   * map's annotations.
   */
  deleteScreenshot: (path: string) => __TAURI_INVOKE<null>("plugin:levels|delete_screenshot", { path }),
  /**
   * Describe what the open level's game runs its console with of how levels are lit and exposed: its `user.ltx` over
   * its shipped defaults; read again where a commit let the shipped ones go.
   */
  describeConsoleDefaults: (sessionId: SessionId) =>
    __TAURI_INVOKE<SessionSnapshot<LevelConsoleDefaults>>("plugin:levels|describe_console_defaults", { sessionId }),
  /**
   * Describe where the open level's map lies over it, as its `level.ltx` places it, or none where it places none; read
   * again where a commit let it go.
   */
  describeLevelMap: (sessionId: SessionId) =>
    __TAURI_INVOKE<SessionSnapshot<LevelMapDescription | null>>("plugin:levels|describe_level_map", { sessionId }),
  /** Describe one of the open level's spawned objects, by its place among them, as describe_spawn_objects numbered it. */
  describeSpawnObject: (sessionId: SessionId, index: number) =>
    __TAURI_INVOKE<SessionSnapshot<LevelSpawnObjectDetails>>("plugin:levels|describe_spawn_object", {
      sessionId,
      index,
    }),
  /**
   * Describe the open level's spawned objects the viewer draws, and the visuals they stand as, reading no visual; an
   * error where the spawn cannot be read.
   */
  describeSpawnObjects: (sessionId: SessionId) =>
    __TAURI_INVOKE<SessionSnapshot<LevelSpawnObjectsDescription>>("plugin:levels|describe_spawn_objects", {
      sessionId,
    }),
  /** Describe where the open level opens, or none for a viewer to decide; decided again where a commit let it go. */
  describeStart: (sessionId: SessionId) =>
    __TAURI_INVOKE<SessionSnapshot<LevelStart | null>>("plugin:levels|describe_start", { sessionId }),
  /**
   * Describe how the renderer draws each entry of the open level's shader table, in its order, so the page names a
   * surface by the shader id a packed surface carries; resolved again where a commit let it go.
   */
  describeSurfaces: (sessionId: SessionId) =>
    __TAURI_INVOKE<SessionSnapshot<Array<XraySurfaceDescriptor>>>("plugin:levels|describe_surfaces", { sessionId }),
  /** Describe what every texture the open level's surfaces bind resolved to; resolved again where a commit let it go. */
  describeTextures: (sessionId: SessionId) =>
    __TAURI_INVOKE<SessionSnapshot<Array<LevelTextureReference>>>("plugin:levels|describe_textures", { sessionId }),
  /**
   * Describe the open level's weather as its engine loads it: the cycles it plays, the effects, what they strike with,
   * the sun table and the level's own overrides.
   */
  describeWeather: (sessionId: SessionId) =>
    __TAURI_INVOKE<SessionSnapshot<LevelWeatherDescription>>("plugin:levels|describe_weather", { sessionId }),
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
  /** The folder the level viewer's views are exported to and imported from by default, made where it is not yet. */
  getViewFolder: () => __TAURI_INVOKE<string>("plugin:levels|get_view_folder"),
  /** Every compiled level the mounted roots hold, loose or archived alike, listed on the pool from a fresh search. */
  listLevels: (roots: XrayRoots) => __TAURI_INVOKE<Array<LevelEntry>>("plugin:levels|list_levels", { roots }),
  /** Select a compiled level and report what it is built out of, without reading any of its geometry. */
  openLevel: (sessionId: SessionId, request: LevelOpenRequest) =>
    __TAURI_INVOKE<SessionSnapshot<SelectedLevelDescription>>("plugin:levels|open_level", { sessionId, request }),
  /**
   * Read any cycle or effect of the game as the open level's engine loads it, for a viewer playing one the level does
   * not offer itself.
   */
  readLevelCycle: (sessionId: SessionId, cycle: WeatherCycleId) =>
    __TAURI_INVOKE<SessionSnapshot<LevelWeatherCycle>>("plugin:levels|read_level_cycle", { sessionId, cycle }),
  /** Read a level view's JSON file as text, for the page to check and import. */
  readView: (path: string) => __TAURI_INVOKE<string>("plugin:levels|read_view", { path }),
  /**
   * Restore the level `tool` holds in the asking window, the newest it opened, without reading the level again: as a
   * page attaches to it, or reads it again after a commit derived it anew. Its parts are read by their own queries.
   */
  restoreLevel: (tool: string) =>
    __TAURI_INVOKE<SessionRestore<SelectedLevelDescription>>("plugin:levels|restore_level", { tool }),
  /**
   * Write a viewport's next finished frame to a PNG file of the level viewer's screenshots folder: the whole viewport at
   * its drawn size, without its overlays or its selection's outline.
   */
  saveScreenshot: (viewport: RenderViewportId, path: string) =>
    __TAURI_INVOKE<null>("plugin:levels|save_screenshot", { viewport, path }),
  /** Write a level view as its JSON text to the JSON file the page names. */
  saveView: (path: string, text: string) => __TAURI_INVOKE<null>("plugin:levels|save_view", { path, text }),
  /**
   * Draw the open level in a viewport, its sectors and textures read by the renderer; no session draws none. Answers
   * the scene drawn, which the viewport's load reports name, or none. The source may resolve the level's surfaces, so
   * it is made on the pool, after the ticket that keeps a level asked for later from being replaced by this one.
   */
  showLevel: (viewport: RenderViewportId, sessionId: string | null) =>
    __TAURI_INVOKE<number | null>("plugin:levels|show_level", { viewport, sessionId }),
  /** Draw the open level's annotations over one of its level maps and write them beside it as an SVG; answers the SVG. */
  writeMapAnnotations: (sessionId: SessionId, request: MapAnnotationsRequest) =>
    __TAURI_INVOKE<SessionSnapshot<string>>("plugin:levels|write_map_annotations", { sessionId, request }),
  /** Write a screenshot's report beside it in the screenshots folder, as its JSON text; answers where it was written. */
  writeScreenshotReport: (path: string, text: string) =>
    __TAURI_INVOKE<string>("plugin:levels|write_screenshot_report", { path, text }),
};
