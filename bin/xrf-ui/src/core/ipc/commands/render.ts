// Auto-generated rust bindings. Do not edit it manually.

import { Channel } from "@tauri-apps/api/core";

import { invoke as __TAURI_INVOKE } from "@/core/ipc/invoke";
import { ETextureSurfaceAlpha, ETextureSurfaceShape, TextureSource, ViewportEvent } from "@/core/ipc/types/xrf-app";
import {
  RenderFramePhases,
  RenderLightsReport,
  RenderLoadDurations,
  RenderMemoryReport,
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
} from "@/core/ipc/types/xrf-renderer";
import { XrayRoots } from "@/core/ipc/types/xrf-vfs";
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
} from "@/core/ipc/types/xrf-world";

/** Commands */
export const renderCommands = {
  /**
   * Start drawing a native viewport into a window, named by its label, telling the page what it costs and where its
   * camera is through `events`.
   */
  attachViewport: (window: string, events: Channel<ViewportEvent>) =>
    __TAURI_INVOKE<RenderViewportId>("plugin:render|attach_viewport", { window, events }),
  /** Ask a viewport's camera to reset or dolly. */
  commandCamera: (viewport: RenderViewportId, command: WorldCameraCommand) =>
    __TAURI_INVOKE<void>("plugin:render|command_camera", { viewport, command }),
  /** Apply settings every native viewport draws with. */
  configure: (settings: RenderSettings) => __TAURI_INVOKE<void>("plugin:render|configure", { settings }),
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
      /** Mean milliseconds of the render thread's own work a frame: recording and submitting, not waiting. */
      cpuTime: number | null;
      /** Where the render thread's time goes a frame, waiting for the window's image and presenting included. */
      phases: RenderFramePhases;
      /** Drawn width, in device pixels. */
      width: number;
      /** Drawn height, in device pixels. */
      height: number;
      /** The scene's width as rendered, smaller than the drawn one where it is upscaled. */
      renderWidth: number;
      /** And its height. */
      renderHeight: number;
      /** The graphics API drawn with. */
      backend: string;
      /** The GPU drawn on. */
      adapter: string;
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
      /** Milliseconds the last sector taken in took to put into the scene, on the render thread. */
      sectorTime: number | null;
      /** What the renderer holds on the GPU. */
      memory: RenderMemoryReport;
    } | null>("plugin:render|describe_frame", { viewport }),
  /**
   * Say how far the level a viewport was last asked to show has loaded, for a caller polling rather than listening; none
   * before its view is made.
   */
  describeLoad: (viewport: RenderViewportId) =>
    __TAURI_INVOKE<{
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
    } | null>("plugin:render|describe_load", { viewport }),
  /** Say what a viewport's level could not draw: drawables the packer left out, sectors and spawned models unread. */
  describeProblems: (viewport: RenderViewportId) =>
    __TAURI_INVOKE<WorldLevelProblems>("plugin:render|describe_problems", { viewport }),
  /** Say what became of every texture a viewport's level samples. */
  describeTextures: (viewport: RenderViewportId) =>
    __TAURI_INVOKE<Array<RenderTextureReport>>("plugin:render|describe_textures", { viewport }),
  /** Stop drawing a viewport; the GPU goes a few seconds after the last one. */
  detachViewport: (viewport: RenderViewportId) => __TAURI_INVOKE<void>("plugin:render|detach_viewport", { viewport }),
  /**
   * Find a spawned object's bounding sphere in a viewport's level, centre then radius in renderer space; none until
   * its model is drawn.
   */
  locateSpawnObject: (viewport: RenderViewportId, object: number) =>
    __TAURI_INVOKE<[number | null, number | null, number | null, number | null] | null>(
      "plugin:render|locate_spawn_object",
      { viewport, object }
    ),
  /** Count what each shader table entry of a viewport's level draws across the sectors resident. */
  measureSurfaces: (viewport: RenderViewportId) =>
    __TAURI_INVOKE<Array<WorldSurfaceGeometry>>("plugin:render|measure_surfaces", { viewport }),
  /** Name what a viewport's level draws under a point, css pixels from its corner, or nothing. */
  pick: (viewport: RenderViewportId, x: number | null, y: number | null) =>
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
    >("plugin:render|pick", { viewport, x, y }),
  /** Stand one viewport's skinned models in a pose. */
  poseModel: (viewport: RenderViewportId, pose: WorldModelPose) =>
    __TAURI_INVOKE<void>("plugin:render|pose_model", { viewport, pose }),
  /** Play a weather ambient effect near a viewport's camera at once, ending the one playing; none plays indoors. */
  playAmbientEffect: (viewport: RenderViewportId) =>
    __TAURI_INVOKE<void>("plugin:render|play_ambient_effect", { viewport }),
  /** Play a weather in a viewport's level from now on: a cycle by name, a keyframe set by hand, or nothing. */
  playWeather: (viewport: RenderViewportId, play: WorldWeatherPlay, transition: EWorldWeatherTransition) =>
    __TAURI_INVOKE<void>("plugin:render|play_weather", { viewport, play, transition }),
  /** Play a weather effect over a viewport's cycle from its clock's time, or end the one playing for none. */
  playWeatherEffect: (viewport: RenderViewportId, name: string | null) =>
    __TAURI_INVOKE<void>("plugin:render|play_weather_effect", { viewport, name }),
  /** Write a viewport's next presented frame to a PNG file, as the renderer drew it rather than as the screen shows it. */
  saveCapture: (viewport: RenderViewportId, path: string) =>
    __TAURI_INVOKE<null>("plugin:render|save_capture", { viewport, path }),
  /** Play a viewport's weather on from a time of day, in seconds since midnight, ending the effect playing. */
  seekWeather: (viewport: RenderViewportId, time: number | null) =>
    __TAURI_INVOKE<void>("plugin:render|seek_weather", { viewport, time }),
  /** Hand a viewport one gesture the page heard over it. */
  sendInput: (viewport: RenderViewportId, event: WorldInputEvent) =>
    __TAURI_INVOKE<void>("plugin:render|send_input", { viewport, event }),
  /** Describe a viewport's camera; described again from the same start, it keeps where it has been moved. */
  setCamera: (viewport: RenderViewportId, camera: WorldCamera) =>
    __TAURI_INVOKE<void>("plugin:render|set_camera", { viewport, camera }),
  /** Set the helpers drawn over one viewport's frame. */
  setOverlays: (viewport: RenderViewportId, overlays: Array<RenderOverlay>) =>
    __TAURI_INVOKE<void>("plugin:render|set_overlays", { viewport, overlays }),
  /** Mark what of one viewport's level is selected, or nothing. */
  setSelection: (
    viewport: RenderViewportId,
    selection: {
      target: RenderSelectionTarget;
      /** sRGB from 0 to 1. */
      color: [number | null, number | null, number | null];
    } | null
  ) => __TAURI_INVOKE<void>("plugin:render|set_selection", { viewport, selection }),
  /** Set what one viewport draws its scene with. */
  setViewOptions: (viewport: RenderViewportId, options: RenderViewOptions) =>
    __TAURI_INVOKE<void>("plugin:render|set_view_options", { viewport, options }),
  /** Place a viewport where its element now is, and say what the page shows around it. */
  setViewportLayout: (viewport: RenderViewportId, layout: RenderViewportLayout) =>
    __TAURI_INVOKE<void>("plugin:render|set_viewport_layout", { viewport, layout }),
  /** Set how a viewport's weather clock runs. */
  setWeatherControl: (viewport: RenderViewportId, control: WorldWeatherControl) =>
    __TAURI_INVOKE<void>("plugin:render|set_weather_control", { viewport, control }),
  /**
   * Set what of one viewport's world plays: the weather's rain, bolts and wind, the campfires, the ambient effects, and
   * which spawn groups stream in.
   */
  setWorldToggles: (viewport: RenderViewportId, toggles: WorldToggles) =>
    __TAURI_INVOKE<void>("plugin:render|set_world_toggles", { viewport, toggles }),
  /** Draw the open level in a viewport, its sectors and textures read by the renderer; no session draws none. */
  showLevel: (viewport: RenderViewportId, sessionId: string | null) =>
    __TAURI_INVOKE<null>("plugin:render|show_level", { viewport, sessionId }),
  /**
   * Draw the open model in a viewport, `detail` down its collapse chain, its textures read by the renderer; no session
   * draws none.
   */
  showModel: (viewport: RenderViewportId, sessionId: string | null, detail: number | null) =>
    __TAURI_INVOKE<null>("plugin:render|show_model", { viewport, sessionId, detail }),
  /** Draw a texture laid on a body in a viewport, its files read by the renderer; no request draws none. */
  showTexture: (
    viewport: RenderViewportId,
    request: {
      source: TextureSource;
      /** The roots the texture is resolved in, as its description was. */
      roots: XrayRoots;
      shape: ETextureSurfaceShape;
      /** How many times the texture repeats across the body, which is how a tiling seam becomes visible. */
      tiling: number | null;
      alpha: ETextureSurfaceAlpha;
      /** Width over height of the texture, which the plane is stretched to. */
      aspect: number | null;
    } | null
  ) => __TAURI_INVOKE<null>("plugin:render|show_texture", { viewport, request }),
};
