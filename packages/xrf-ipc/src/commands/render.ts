// Auto-generated rust bindings. Do not edit it manually.

import { Channel } from "@tauri-apps/api/core";

import { invoke as __TAURI_INVOKE } from "#/invoke";
import { IIpcCommandSignature } from "#/signature";
import { RenderPickRequest, ViewportEvent } from "#/types/xrf-app-render";
import {
  RenderAntialiasing,
  RenderBackend,
  RenderBackendAvailability,
  RenderDlssAvailability,
  RenderFramePhases,
  RenderGraphReport,
  RenderGrassReport,
  RenderGroupCost,
  RenderLibraries,
  RenderLightsReport,
  RenderLoadDurations,
  RenderMemoryReport,
  RenderMeterReading,
  RenderOverlay,
  RenderParticlesReport,
  RenderPassCost,
  RenderSelectionTarget,
  RenderSettings,
  RenderStaticReport,
  RenderTextureReport,
  RenderViewOptions,
  RenderViewportId,
  RenderViewportLayout,
} from "#/types/xrf-renderer";
import {
  EWorldWeatherTransition,
  WorldCamera,
  WorldCameraCommand,
  WorldInputEvent,
  WorldLevelProblems,
  WorldModelPose,
  WorldSurfaceGeometry,
  WorldToggles,
  WorldWeatherControl,
  WorldWeatherPlay,
} from "#/types/xrf-world";

/** The wire name of every command of the `render` plugin. */
export enum ERenderCommand {
  ATTACH_VIEWPORT = "plugin:render|attach_viewport",
  COMMAND_CAMERA = "plugin:render|command_camera",
  CONFIGURE = "plugin:render|configure",
  DESCRIBE_FRAME = "plugin:render|describe_frame",
  DESCRIBE_LIBRARIES = "plugin:render|describe_libraries",
  DESCRIBE_LOAD = "plugin:render|describe_load",
  DESCRIBE_PROBLEMS = "plugin:render|describe_problems",
  DESCRIBE_TEXTURE_LOADS = "plugin:render|describe_texture_loads",
  DETACH_VIEWPORT = "plugin:render|detach_viewport",
  LIST_BACKENDS = "plugin:render|list_backends",
  LOCATE_SPAWN_OBJECT = "plugin:render|locate_spawn_object",
  MARK_PROFILE = "plugin:render|mark_profile",
  MEASURE_SURFACES = "plugin:render|measure_surfaces",
  PICK = "plugin:render|pick",
  POSE_MODEL = "plugin:render|pose_model",
  PLAY_AMBIENT_EFFECT = "plugin:render|play_ambient_effect",
  PLAY_WEATHER = "plugin:render|play_weather",
  PLAY_WEATHER_EFFECT = "plugin:render|play_weather_effect",
  SAVE_CAPTURE = "plugin:render|save_capture",
  SEEK_WEATHER = "plugin:render|seek_weather",
  SEND_INPUT = "plugin:render|send_input",
  SET_CAMERA = "plugin:render|set_camera",
  SET_OVERLAYS = "plugin:render|set_overlays",
  SET_SELECTION = "plugin:render|set_selection",
  SET_VIEW_OPTIONS = "plugin:render|set_view_options",
  SET_VIEWPORT_LAYOUT = "plugin:render|set_viewport_layout",
  SET_WEATHER_CONTROL = "plugin:render|set_weather_control",
  SET_WORLD_TOGGLES = "plugin:render|set_world_toggles",
}

/** Commands */
export const renderCommands = {
  /**
   * Start drawing a native viewport into the calling window, telling the page what it costs and where its camera is
   * through `events`.
   */
  attachViewport: (events: Channel<ViewportEvent>) =>
    __TAURI_INVOKE<RenderViewportId>(ERenderCommand.ATTACH_VIEWPORT, { events }),
  /** Ask a viewport's camera to reset or dolly. */
  commandCamera: (viewport: RenderViewportId, command: WorldCameraCommand) =>
    __TAURI_INVOKE<void>(ERenderCommand.COMMAND_CAMERA, { viewport, command }),
  /** Apply settings every native viewport draws with. */
  configure: (settings: RenderSettings) => __TAURI_INVOKE<void>(ERenderCommand.CONFIGURE, { settings }),
  /**
   * Say what a viewport's frames cost when it last reported them, as its `Frame` events do, for a caller polling rather
   * than listening; none before its first report.
   */
  describeFrame: (viewport: RenderViewportId) =>
    __TAURI_INVOKE<{
      /** Frames presented a second over the reported span. */
      framesPerSecond: number | null;
      /** Mean milliseconds between presented frames. */
      frameTime: number | null;
      /** Longest milliseconds between two presented frames. */
      frameTimeMax: number | null;
      /** Milliseconds between each two presented frames over the span, in order. */
      frameTimes: Array<number | null>;
      /** Mean milliseconds of the render thread's own work a frame: recording and submitting, not waiting. */
      cpuTime: number | null;
      /** Milliseconds of the render thread's own work each frame over the span, in order. */
      cpuTimes: Array<number | null>;
      /**
       * Mean milliseconds the GPU spent on a frame from its first command to its last, whether or not its passes are
       * timed; none where the device cannot time it.
       */
      gpuTime: number | null;
      /** Milliseconds the GPU spent on each frame read back over the span, in order; each comes back a few frames late. */
      gpuTimes: Array<number | null>;
      /** Where the render thread's time goes a frame, waiting for the window's image and presenting included. */
      phases: RenderFramePhases;
      /** What each encode group cost the render thread a frame, in the order the frame records them. */
      groups: Array<RenderGroupCost>;
      /** Drawn width, in device pixels. */
      width: number;
      /** Drawn height, in device pixels. */
      height: number;
      /** The scene's width as rendered, smaller than the drawn one where it is upscaled. */
      renderWidth: number;
      /** And its height. */
      renderHeight: number;
      /** The width the frame is resolved at before it is rescaled to the drawn one. */
      outputWidth: number;
      /** And its height. */
      outputHeight: number;
      /** What smooths the frame, and upscales it where it resolves temporally. */
      antialiasing: RenderAntialiasing;
      /** Whether DLSS reconstructs rays as it upscales the frame. */
      isRayReconstructed: boolean;
      /** The graphics API drawn with. */
      backend: RenderBackend;
      /** The GPU drawn on. */
      adapter: string;
      /**
       * Whether it traces rays, by hardware ray query: what the reflections' and the water's tracing of what the screen
       * misses needs.
       */
      hasRayQueries: boolean;
      /** Whether it upscales with DLSS, and what it lacks where it does not. */
      dlss: RenderDlssAvailability;
      /** Whether it reconstructs rays with DLSS, and what it lacks where it does not. */
      rayReconstruction: RenderDlssAvailability;
      /** Whether its passes were timed on the GPU over the span. */
      isGpuTimed: boolean;
      /** What each pass cost on the GPU, in frame order; none while untimed. */
      passes: Array<RenderPassCost>;
      /** The level's static draws' pools and cull, empty without a level. */
      staticDraws: RenderStaticReport;
      /** The level's local lights, empty without a level. */
      lights: RenderLightsReport;
      /** The level's particle systems, empty without a level. */
      particles: RenderParticlesReport;
      /** The view's grass, empty without a level or grass. */
      grass: RenderGrassReport;
      /** Milliseconds the last sector taken in took to put into the scene, on the render thread. */
      sectorTime: number | null;
      /** What the renderer holds on the GPU. */
      memory: RenderMemoryReport;
      /** What the view's spot meter read, a frame or more late; none while it does not meter. */
      meter: RenderMeterReading | null;
      /** What the frame graph made of its latest frame, none before its first. */
      graph: RenderGraphReport | null;
    } | null>(ERenderCommand.DESCRIBE_FRAME, { viewport }),
  /** Report the versions of the graphics libraries the renderer ships with; currently only the DLSS runtime. */
  describeLibraries: () => __TAURI_INVOKE<RenderLibraries>(ERenderCommand.DESCRIBE_LIBRARIES),
  /**
   * Say how far the level a viewport was last asked to show has loaded, for a caller polling rather than listening; none
   * before its view is made.
   */
  describeLoad: (viewport: RenderViewportId) =>
    __TAURI_INVOKE<{
      /** The scene it reports on, as showing the level named it. */
      scene: number;
      /** Sectors resident on the GPU. */
      sectors: number;
      /** Sectors the level has. */
      sectorsTotal: number;
      /** Bytes of the sectors resident, packed. */
      bytes: number;
      /** Textures uploaded or given up on. */
      textures: number;
      /** Textures the level samples: its scene's, its lights' projectors and its particles'. */
      texturesTotal: number;
      /**
       * Whether everything is resident, so the level draws as it will: every sector, the spawn, grass, lights and
       * particles read, and every texture settled.
       */
      isReady: boolean;
      /** How long each part took to finish, timed from when the level began opening. */
      durations: RenderLoadDurations;
    } | null>(ERenderCommand.DESCRIBE_LOAD, { viewport }),
  /** Say what a viewport's level could not draw: drawables the packer left out, sectors and spawned models unread. */
  describeProblems: (viewport: RenderViewportId) =>
    __TAURI_INVOKE<WorldLevelProblems>(ERenderCommand.DESCRIBE_PROBLEMS, { viewport }),
  /** Say what became of every texture a viewport's level samples. */
  describeTextureLoads: (viewport: RenderViewportId) =>
    __TAURI_INVOKE<Array<RenderTextureReport>>(ERenderCommand.DESCRIBE_TEXTURE_LOADS, { viewport }),
  /** Stop drawing a viewport; the GPU goes a few seconds after the last one. */
  detachViewport: (viewport: RenderViewportId) => __TAURI_INVOKE<void>(ERenderCommand.DETACH_VIEWPORT, { viewport }),
  /**
   * Say which graphics backends the renderer can draw with on this machine, and on which GPU, for the settings to offer
   * only those; each is probed off the render thread.
   */
  listBackends: () => __TAURI_INVOKE<Array<RenderBackendAvailability>>(ERenderCommand.LIST_BACKENDS),
  /**
   * Find a spawned object's bounding sphere in a viewport's level, centre then radius in renderer space; none until
   * its model is drawn.
   */
  locateSpawnObject: (viewport: RenderViewportId, object: number) =>
    __TAURI_INVOKE<[number | null, number | null, number | null, number | null] | null>(
      ERenderCommand.LOCATE_SPAWN_OBJECT,
      { viewport, object }
    ),
  /** Put a message on a Tracy capture's timeline, as a driver marks each measured scene; nothing in a build without Tracy. */
  markProfile: (text: string) => __TAURI_INVOKE<void>(ERenderCommand.MARK_PROFILE, { text }),
  /** Count what each shader table entry of a viewport's level draws across the sectors resident. */
  measureSurfaces: (viewport: RenderViewportId) =>
    __TAURI_INVOKE<Array<WorldSurfaceGeometry>>(ERenderCommand.MEASURE_SURFACES, { viewport }),
  /**
   * Name what a viewport's level draws under a point, or nothing; glass is the nearest thing a click meets unless the
   * pick passes it by.
   */
  pick: (viewport: RenderViewportId, request: RenderPickRequest) =>
    __TAURI_INVOKE<
      /** A surface the level compiled. */
      | {
          kind: "surface";
          sector: number;
          /** The shader table entry drawing it. */
          shaderId: number;
          /** The sector's instanced mesh it is one place of, or none for its baked geometry. */
          mesh: number | null;
          /** Which place of the mesh it is, or none for the baked geometry. */
          place: number | null;
          /** Whether it is a clump of trees drawn as its impostor. */
          isImpostor: boolean;
          point: [number | null, number | null, number | null];
        }
      /** An object the level's spawn places. */
      | {
          kind: "spawn";
          /** Its index among the level's spawned objects. */
          object: number;
          point: [number | null, number | null, number | null];
        }
      | null
    >(ERenderCommand.PICK, { viewport, request }),
  /** Stand one viewport's skinned models in a pose. */
  poseModel: (viewport: RenderViewportId, pose: WorldModelPose) =>
    __TAURI_INVOKE<void>(ERenderCommand.POSE_MODEL, { viewport, pose }),
  /** Play a weather ambient effect near a viewport's camera at once, ending the one playing; none plays indoors. */
  playAmbientEffect: (viewport: RenderViewportId) =>
    __TAURI_INVOKE<void>(ERenderCommand.PLAY_AMBIENT_EFFECT, { viewport }),
  /** Play a weather in a viewport's level from now on: a cycle by name, a keyframe set by hand, or nothing. */
  playWeather: (viewport: RenderViewportId, play: WorldWeatherPlay, transition: EWorldWeatherTransition) =>
    __TAURI_INVOKE<void>(ERenderCommand.PLAY_WEATHER, { viewport, play, transition }),
  /** Play a weather effect over a viewport's cycle from its clock's time, or end the one playing for none. */
  playWeatherEffect: (viewport: RenderViewportId, name: string | null) =>
    __TAURI_INVOKE<void>(ERenderCommand.PLAY_WEATHER_EFFECT, { viewport, name }),
  /** Write a viewport's next presented frame to a PNG file, as the renderer drew it rather than as the screen shows it. */
  saveCapture: (viewport: RenderViewportId, path: string) =>
    __TAURI_INVOKE<null>(ERenderCommand.SAVE_CAPTURE, { viewport, path }),
  /** Play a viewport's weather on from a time of day, in seconds since midnight, ending the effect playing. */
  seekWeather: (viewport: RenderViewportId, time: number | null) =>
    __TAURI_INVOKE<void>(ERenderCommand.SEEK_WEATHER, { viewport, time }),
  /** Hand a viewport one gesture the page heard over it. */
  sendInput: (viewport: RenderViewportId, event: WorldInputEvent) =>
    __TAURI_INVOKE<void>(ERenderCommand.SEND_INPUT, { viewport, event }),
  /** Describe a viewport's camera; described again from the same start, it keeps where it has been moved. */
  setCamera: (viewport: RenderViewportId, camera: WorldCamera) =>
    __TAURI_INVOKE<void>(ERenderCommand.SET_CAMERA, { viewport, camera }),
  /** Set the helpers drawn over one viewport's frame. */
  setOverlays: (viewport: RenderViewportId, overlays: Array<RenderOverlay>) =>
    __TAURI_INVOKE<void>(ERenderCommand.SET_OVERLAYS, { viewport, overlays }),
  /** Mark what of one viewport's level is selected, or nothing. */
  setSelection: (
    viewport: RenderViewportId,
    selection: {
      target: RenderSelectionTarget;
      /** sRGB from 0 to 1. */
      color: [number | null, number | null, number | null];
    } | null
  ) => __TAURI_INVOKE<void>(ERenderCommand.SET_SELECTION, { viewport, selection }),
  /** Set what one viewport draws its scene with. */
  setViewOptions: (viewport: RenderViewportId, options: RenderViewOptions) =>
    __TAURI_INVOKE<void>(ERenderCommand.SET_VIEW_OPTIONS, { viewport, options }),
  /** Place a viewport where its element now is, and say what the page shows around it. */
  setViewportLayout: (viewport: RenderViewportId, layout: RenderViewportLayout) =>
    __TAURI_INVOKE<void>(ERenderCommand.SET_VIEWPORT_LAYOUT, { viewport, layout }),
  /** Set how a viewport's weather clock runs. */
  setWeatherControl: (viewport: RenderViewportId, control: WorldWeatherControl) =>
    __TAURI_INVOKE<void>(ERenderCommand.SET_WEATHER_CONTROL, { viewport, control }),
  /**
   * Set what of one viewport's world plays: the weather's rain, bolts and wind, the campfires, the ambient effects, and
   * which spawn groups stream in.
   */
  setWorldToggles: (viewport: RenderViewportId, toggles: WorldToggles) =>
    __TAURI_INVOKE<void>(ERenderCommand.SET_WORLD_TOGGLES, { viewport, toggles }),
};

/** What each command of the `render` plugin takes and resolves to, keyed by its wire name. */
export interface IRenderCommandSignatures {
  [ERenderCommand.ATTACH_VIEWPORT]: IIpcCommandSignature<typeof renderCommands.attachViewport, ["events"]>;
  [ERenderCommand.COMMAND_CAMERA]: IIpcCommandSignature<typeof renderCommands.commandCamera, ["viewport", "command"]>;
  [ERenderCommand.CONFIGURE]: IIpcCommandSignature<typeof renderCommands.configure, ["settings"]>;
  [ERenderCommand.DESCRIBE_FRAME]: IIpcCommandSignature<typeof renderCommands.describeFrame, ["viewport"]>;
  [ERenderCommand.DESCRIBE_LIBRARIES]: IIpcCommandSignature<typeof renderCommands.describeLibraries, []>;
  [ERenderCommand.DESCRIBE_LOAD]: IIpcCommandSignature<typeof renderCommands.describeLoad, ["viewport"]>;
  [ERenderCommand.DESCRIBE_PROBLEMS]: IIpcCommandSignature<typeof renderCommands.describeProblems, ["viewport"]>;
  [ERenderCommand.DESCRIBE_TEXTURE_LOADS]: IIpcCommandSignature<
    typeof renderCommands.describeTextureLoads,
    ["viewport"]
  >;
  [ERenderCommand.DETACH_VIEWPORT]: IIpcCommandSignature<typeof renderCommands.detachViewport, ["viewport"]>;
  [ERenderCommand.LIST_BACKENDS]: IIpcCommandSignature<typeof renderCommands.listBackends, []>;
  [ERenderCommand.LOCATE_SPAWN_OBJECT]: IIpcCommandSignature<
    typeof renderCommands.locateSpawnObject,
    ["viewport", "object"]
  >;
  [ERenderCommand.MARK_PROFILE]: IIpcCommandSignature<typeof renderCommands.markProfile, ["text"]>;
  [ERenderCommand.MEASURE_SURFACES]: IIpcCommandSignature<typeof renderCommands.measureSurfaces, ["viewport"]>;
  [ERenderCommand.PICK]: IIpcCommandSignature<typeof renderCommands.pick, ["viewport", "request"]>;
  [ERenderCommand.POSE_MODEL]: IIpcCommandSignature<typeof renderCommands.poseModel, ["viewport", "pose"]>;
  [ERenderCommand.PLAY_AMBIENT_EFFECT]: IIpcCommandSignature<typeof renderCommands.playAmbientEffect, ["viewport"]>;
  [ERenderCommand.PLAY_WEATHER]: IIpcCommandSignature<
    typeof renderCommands.playWeather,
    ["viewport", "play", "transition"]
  >;
  [ERenderCommand.PLAY_WEATHER_EFFECT]: IIpcCommandSignature<
    typeof renderCommands.playWeatherEffect,
    ["viewport", "name"]
  >;
  [ERenderCommand.SAVE_CAPTURE]: IIpcCommandSignature<typeof renderCommands.saveCapture, ["viewport", "path"]>;
  [ERenderCommand.SEEK_WEATHER]: IIpcCommandSignature<typeof renderCommands.seekWeather, ["viewport", "time"]>;
  [ERenderCommand.SEND_INPUT]: IIpcCommandSignature<typeof renderCommands.sendInput, ["viewport", "event"]>;
  [ERenderCommand.SET_CAMERA]: IIpcCommandSignature<typeof renderCommands.setCamera, ["viewport", "camera"]>;
  [ERenderCommand.SET_OVERLAYS]: IIpcCommandSignature<typeof renderCommands.setOverlays, ["viewport", "overlays"]>;
  [ERenderCommand.SET_SELECTION]: IIpcCommandSignature<typeof renderCommands.setSelection, ["viewport", "selection"]>;
  [ERenderCommand.SET_VIEW_OPTIONS]: IIpcCommandSignature<
    typeof renderCommands.setViewOptions,
    ["viewport", "options"]
  >;
  [ERenderCommand.SET_VIEWPORT_LAYOUT]: IIpcCommandSignature<
    typeof renderCommands.setViewportLayout,
    ["viewport", "layout"]
  >;
  [ERenderCommand.SET_WEATHER_CONTROL]: IIpcCommandSignature<
    typeof renderCommands.setWeatherControl,
    ["viewport", "control"]
  >;
  [ERenderCommand.SET_WORLD_TOGGLES]: IIpcCommandSignature<
    typeof renderCommands.setWorldToggles,
    ["viewport", "toggles"]
  >;
}
