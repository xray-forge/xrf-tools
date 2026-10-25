// Auto-generated rust bindings. Do not edit it manually.

import { Channel } from "@tauri-apps/api/core";

import { invoke as __TAURI_INVOKE } from "#/invoke";
import { IIpcCommandSignature } from "#/signature";
import { SessionId, SessionRestore, SessionSnapshot } from "#/types/xrf-app-core";
import {
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
  MapAnnotationsWriteRequest,
  MapCaptureRequest,
  MapExportRequest,
  SelectedLevelDescription,
} from "#/types/xrf-app-levels";
import { CaptureOutcome } from "#/types/xrf-app-render";
import { WeatherCycleId } from "#/types/xrf-environment";
import { JobProgress } from "#/types/xrf-job";
import { XraySurfaceDescriptor } from "#/types/xrf-material";
import { RenderViewportId } from "#/types/xrf-renderer";
import { XrayRoots } from "#/types/xrf-vfs";

/** The wire name of every command of the `levels` plugin. */
export enum ELevelsCommand {
  CAPTURE_CUBE = "plugin:levels|capture_cube",
  CAPTURE_MAP = "plugin:levels|capture_map",
  CLOSE_LEVEL = "plugin:levels|close_level",
  COPY_SCREENSHOT = "plugin:levels|copy_screenshot",
  DELETE_SCREENSHOT = "plugin:levels|delete_screenshot",
  DESCRIBE_CONSOLE_DEFAULTS = "plugin:levels|describe_console_defaults",
  DESCRIBE_LEVEL_MAP = "plugin:levels|describe_level_map",
  DESCRIBE_SPAWN_OBJECT = "plugin:levels|describe_spawn_object",
  DESCRIBE_SPAWN_OBJECTS = "plugin:levels|describe_spawn_objects",
  DESCRIBE_START = "plugin:levels|describe_start",
  DESCRIBE_SURFACES = "plugin:levels|describe_surfaces",
  DESCRIBE_TEXTURES = "plugin:levels|describe_textures",
  DESCRIBE_WEATHER = "plugin:levels|describe_weather",
  EXPORT_CUBE = "plugin:levels|export_cube",
  EXPORT_MAP = "plugin:levels|export_map",
  FIND_SCREENSHOT_PATH = "plugin:levels|find_screenshot_path",
  GET_VIEW_FOLDER = "plugin:levels|get_view_folder",
  LIST_LEVELS = "plugin:levels|list_levels",
  OPEN_LEVEL = "plugin:levels|open_level",
  READ_LEVEL_CYCLE = "plugin:levels|read_level_cycle",
  READ_VIEW = "plugin:levels|read_view",
  RESTORE_LEVEL = "plugin:levels|restore_level",
  SAVE_SCREENSHOT = "plugin:levels|save_screenshot",
  SAVE_VIEW = "plugin:levels|save_view",
  SHOW_LEVEL = "plugin:levels|show_level",
  WRITE_MAP_ANNOTATIONS = "plugin:levels|write_map_annotations",
  WRITE_SCREENSHOT_REPORT = "plugin:levels|write_screenshot_report",
}

/** Commands */
export const levelsCommands = {
  /** Capture a cubemap around a viewport's camera face by face and write it to a PNG file as a horizontal cross. */
  captureCube: (request: CubeCaptureRequest, jobId: string, progress: Channel<JobProgress>) =>
    __TAURI_INVOKE<CaptureOutcome>(ELevelsCommand.CAPTURE_CUBE, { request, jobId, progress }),
  /** Capture a level map in a viewport tile by tile and write it to a PNG file with its coverage in alpha. */
  captureMap: (request: MapCaptureRequest, jobId: string, progress: Channel<JobProgress>) =>
    __TAURI_INVOKE<CaptureOutcome>(ELevelsCommand.CAPTURE_MAP, { request, jobId, progress }),
  /** Close the levels `session_ids` name, each once, unbinding it from its scope. */
  closeLevel: (sessionIds: Array<SessionId>) => __TAURI_INVOKE<void>(ELevelsCommand.CLOSE_LEVEL, { sessionIds }),
  /**
   * Copy a screenshot of the screenshots folder to where the page names, as Save as does; read whole first, so a failed
   * copy leaves no partial file and keeps any file it would have replaced.
   */
  copyScreenshot: (path: string, target: string) =>
    __TAURI_INVOKE<null>(ELevelsCommand.COPY_SCREENSHOT, { path, target }),
  /**
   * Delete a screenshot of the screenshots folder for good, with what stands beside it: a screenshot's report, a level
   * map's annotations.
   */
  deleteScreenshot: (path: string) => __TAURI_INVOKE<null>(ELevelsCommand.DELETE_SCREENSHOT, { path }),
  /**
   * Describe what the open level's game runs its console with of how levels are lit and exposed: its `user.ltx` over
   * its shipped defaults; read again where a commit let the shipped ones go.
   */
  describeConsoleDefaults: (sessionId: SessionId) =>
    __TAURI_INVOKE<SessionSnapshot<LevelConsoleDefaults>>(ELevelsCommand.DESCRIBE_CONSOLE_DEFAULTS, { sessionId }),
  /**
   * Describe where the open level's map lies over it, as its `level.ltx` places it, or none where it places none; read
   * again where a commit let it go.
   */
  describeLevelMap: (sessionId: SessionId) =>
    __TAURI_INVOKE<SessionSnapshot<LevelMapDescription | null>>(ELevelsCommand.DESCRIBE_LEVEL_MAP, { sessionId }),
  /** Describe one of the open level's spawned objects, by its place among them, as describe_spawn_objects numbered it. */
  describeSpawnObject: (sessionId: SessionId, index: number) =>
    __TAURI_INVOKE<SessionSnapshot<LevelSpawnObjectDetails>>(ELevelsCommand.DESCRIBE_SPAWN_OBJECT, {
      sessionId,
      index,
    }),
  /**
   * Describe the open level's spawned objects the viewer draws, and the visuals they stand as, reading no visual; an
   * error where the spawn cannot be read.
   */
  describeSpawnObjects: (sessionId: SessionId) =>
    __TAURI_INVOKE<SessionSnapshot<LevelSpawnObjectsDescription>>(ELevelsCommand.DESCRIBE_SPAWN_OBJECTS, { sessionId }),
  /** Describe where the open level opens, or none for a viewer to decide; decided again where a commit let it go. */
  describeStart: (sessionId: SessionId) =>
    __TAURI_INVOKE<SessionSnapshot<LevelStart | null>>(ELevelsCommand.DESCRIBE_START, { sessionId }),
  /**
   * Describe how the renderer draws each entry of the open level's shader table, in its order, so the page names a
   * surface by the shader id a packed surface carries; resolved again where a commit let it go.
   */
  describeSurfaces: (sessionId: SessionId) =>
    __TAURI_INVOKE<SessionSnapshot<Array<XraySurfaceDescriptor>>>(ELevelsCommand.DESCRIBE_SURFACES, { sessionId }),
  /** Describe what every texture the open level's surfaces bind resolved to; resolved again where a commit let it go. */
  describeTextures: (sessionId: SessionId) =>
    __TAURI_INVOKE<SessionSnapshot<Array<LevelTextureReference>>>(ELevelsCommand.DESCRIBE_TEXTURES, { sessionId }),
  /**
   * Describe the open level's weather as its engine loads it: the cycles it plays, the effects, what they strike with,
   * the sun table and the level's own overrides.
   */
  describeWeather: (sessionId: SessionId) =>
    __TAURI_INVOKE<SessionSnapshot<LevelWeatherDescription>>(ELevelsCommand.DESCRIBE_WEATHER, { sessionId }),
  /** Write a cubemap of the screenshots folder again as a cube DDS, six TGAs or six PNGs; answers the files written. */
  exportCube: (request: CubeExportRequest) => __TAURI_INVOKE<Array<string>>(ELevelsCommand.EXPORT_CUBE, { request }),
  /**
   * Write a level map of the screenshots folder again as a PNG, a TGA or a DDS, or its annotations alone as an SVG;
   * answers its pixels across and down.
   */
  exportMap: (request: MapExportRequest) => __TAURI_INVOKE<[number, number]>(ELevelsCommand.EXPORT_MAP, { request }),
  /**
   * The first free path for a screenshot named `name` in the screenshots folder, `-2`, `-3` and on added for one taken
   * within the same second.
   */
  findScreenshotPath: (name: string) => __TAURI_INVOKE<string>(ELevelsCommand.FIND_SCREENSHOT_PATH, { name }),
  /** The folder the level viewer's views are exported to and imported from by default, made where it is not yet. */
  getViewFolder: () => __TAURI_INVOKE<string>(ELevelsCommand.GET_VIEW_FOLDER),
  /** Every compiled level the mounted roots hold, loose or archived alike, listed on the pool from a fresh search. */
  listLevels: (roots: XrayRoots) => __TAURI_INVOKE<Array<LevelEntry>>(ELevelsCommand.LIST_LEVELS, { roots }),
  /** Select a compiled level and report what it is built out of, without reading any of its geometry. */
  openLevel: (sessionId: SessionId, request: LevelOpenRequest) =>
    __TAURI_INVOKE<SessionSnapshot<SelectedLevelDescription>>(ELevelsCommand.OPEN_LEVEL, { sessionId, request }),
  /**
   * Read any cycle or effect of the game as the open level's engine loads it, for a viewer playing one the level does
   * not offer itself.
   */
  readLevelCycle: (sessionId: SessionId, cycle: WeatherCycleId) =>
    __TAURI_INVOKE<SessionSnapshot<LevelWeatherCycle>>(ELevelsCommand.READ_LEVEL_CYCLE, { sessionId, cycle }),
  /** Read a level view's JSON file as text, for the page to check and import. */
  readView: (path: string) => __TAURI_INVOKE<string>(ELevelsCommand.READ_VIEW, { path }),
  /**
   * Restore the level `tool` holds in the asking window, the newest it opened, without reading the level again: as a
   * page attaches to it, or reads it again after a commit derived it anew. Its parts are read by their own queries.
   */
  restoreLevel: (tool: string) =>
    __TAURI_INVOKE<SessionRestore<SelectedLevelDescription>>(ELevelsCommand.RESTORE_LEVEL, { tool }),
  /**
   * Write a viewport's next finished frame to a PNG file of the level viewer's screenshots folder: the whole viewport at
   * its drawn size, without its overlays or its selection's outline.
   */
  saveScreenshot: (viewport: RenderViewportId, path: string) =>
    __TAURI_INVOKE<null>(ELevelsCommand.SAVE_SCREENSHOT, { viewport, path }),
  /** Write a level view as its JSON text to the JSON file the page names. */
  saveView: (path: string, text: string) => __TAURI_INVOKE<null>(ELevelsCommand.SAVE_VIEW, { path, text }),
  /**
   * Draw the open level in a viewport, its sectors and textures read by the renderer; no session draws none. Answers
   * the scene drawn, which the viewport's load reports name, or none. The source may resolve the level's surfaces, so
   * it is made on the pool, after the ticket that keeps a level asked for later from being replaced by this one.
   */
  showLevel: (viewport: RenderViewportId, sessionId: SessionId | null) =>
    __TAURI_INVOKE<number | null>(ELevelsCommand.SHOW_LEVEL, { viewport, sessionId }),
  /** Draw the open level's annotations over one of its level maps and write them beside it as an SVG; answers the SVG. */
  writeMapAnnotations: (sessionId: SessionId, request: MapAnnotationsWriteRequest) =>
    __TAURI_INVOKE<SessionSnapshot<string>>(ELevelsCommand.WRITE_MAP_ANNOTATIONS, { sessionId, request }),
  /** Write a screenshot's report beside it in the screenshots folder, as its JSON text; answers where it was written. */
  writeScreenshotReport: (path: string, text: string) =>
    __TAURI_INVOKE<string>(ELevelsCommand.WRITE_SCREENSHOT_REPORT, { path, text }),
};

/** What each command of the `levels` plugin takes and resolves to, keyed by its wire name. */
export interface ILevelsCommandSignatures {
  [ELevelsCommand.CAPTURE_CUBE]: IIpcCommandSignature<
    typeof levelsCommands.captureCube,
    ["request", "jobId", "progress"]
  >;
  [ELevelsCommand.CAPTURE_MAP]: IIpcCommandSignature<
    typeof levelsCommands.captureMap,
    ["request", "jobId", "progress"]
  >;
  [ELevelsCommand.CLOSE_LEVEL]: IIpcCommandSignature<typeof levelsCommands.closeLevel, ["sessionIds"]>;
  [ELevelsCommand.COPY_SCREENSHOT]: IIpcCommandSignature<typeof levelsCommands.copyScreenshot, ["path", "target"]>;
  [ELevelsCommand.DELETE_SCREENSHOT]: IIpcCommandSignature<typeof levelsCommands.deleteScreenshot, ["path"]>;
  [ELevelsCommand.DESCRIBE_CONSOLE_DEFAULTS]: IIpcCommandSignature<
    typeof levelsCommands.describeConsoleDefaults,
    ["sessionId"]
  >;
  [ELevelsCommand.DESCRIBE_LEVEL_MAP]: IIpcCommandSignature<typeof levelsCommands.describeLevelMap, ["sessionId"]>;
  [ELevelsCommand.DESCRIBE_SPAWN_OBJECT]: IIpcCommandSignature<
    typeof levelsCommands.describeSpawnObject,
    ["sessionId", "index"]
  >;
  [ELevelsCommand.DESCRIBE_SPAWN_OBJECTS]: IIpcCommandSignature<
    typeof levelsCommands.describeSpawnObjects,
    ["sessionId"]
  >;
  [ELevelsCommand.DESCRIBE_START]: IIpcCommandSignature<typeof levelsCommands.describeStart, ["sessionId"]>;
  [ELevelsCommand.DESCRIBE_SURFACES]: IIpcCommandSignature<typeof levelsCommands.describeSurfaces, ["sessionId"]>;
  [ELevelsCommand.DESCRIBE_TEXTURES]: IIpcCommandSignature<typeof levelsCommands.describeTextures, ["sessionId"]>;
  [ELevelsCommand.DESCRIBE_WEATHER]: IIpcCommandSignature<typeof levelsCommands.describeWeather, ["sessionId"]>;
  [ELevelsCommand.EXPORT_CUBE]: IIpcCommandSignature<typeof levelsCommands.exportCube, ["request"]>;
  [ELevelsCommand.EXPORT_MAP]: IIpcCommandSignature<typeof levelsCommands.exportMap, ["request"]>;
  [ELevelsCommand.FIND_SCREENSHOT_PATH]: IIpcCommandSignature<typeof levelsCommands.findScreenshotPath, ["name"]>;
  [ELevelsCommand.GET_VIEW_FOLDER]: IIpcCommandSignature<typeof levelsCommands.getViewFolder, []>;
  [ELevelsCommand.LIST_LEVELS]: IIpcCommandSignature<typeof levelsCommands.listLevels, ["roots"]>;
  [ELevelsCommand.OPEN_LEVEL]: IIpcCommandSignature<typeof levelsCommands.openLevel, ["sessionId", "request"]>;
  [ELevelsCommand.READ_LEVEL_CYCLE]: IIpcCommandSignature<typeof levelsCommands.readLevelCycle, ["sessionId", "cycle"]>;
  [ELevelsCommand.READ_VIEW]: IIpcCommandSignature<typeof levelsCommands.readView, ["path"]>;
  [ELevelsCommand.RESTORE_LEVEL]: IIpcCommandSignature<typeof levelsCommands.restoreLevel, ["tool"]>;
  [ELevelsCommand.SAVE_SCREENSHOT]: IIpcCommandSignature<typeof levelsCommands.saveScreenshot, ["viewport", "path"]>;
  [ELevelsCommand.SAVE_VIEW]: IIpcCommandSignature<typeof levelsCommands.saveView, ["path", "text"]>;
  [ELevelsCommand.SHOW_LEVEL]: IIpcCommandSignature<typeof levelsCommands.showLevel, ["viewport", "sessionId"]>;
  [ELevelsCommand.WRITE_MAP_ANNOTATIONS]: IIpcCommandSignature<
    typeof levelsCommands.writeMapAnnotations,
    ["sessionId", "request"]
  >;
  [ELevelsCommand.WRITE_SCREENSHOT_REPORT]: IIpcCommandSignature<
    typeof levelsCommands.writeScreenshotReport,
    ["path", "text"]
  >;
}
