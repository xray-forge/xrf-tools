// Auto-generated rust bindings. Do not edit it manually.

import { Channel } from "@tauri-apps/api/core";

import { invoke as __TAURI_INVOKE } from "@/core/ipc/invoke";
import {
  ERenderWeatherTransition,
  RenderCamera,
  RenderCameraCommand,
  RenderInputEvent,
  RenderLevelProblems,
  RenderModelPose,
  RenderOverlay,
  RenderSettings,
  RenderSurfaceGeometry,
  RenderTextureReport,
  RenderViewOptions,
  RenderViewportEvent,
  RenderViewportId,
  RenderViewportLayout,
  RenderWeatherControl,
  RenderWeatherPlay,
} from "@/core/ipc/types/xrf-renderer";

/** Commands */
export const renderCommands = {
  /**
   * Start drawing a native viewport into a window, named by its label, telling the page what it costs and where its
   * camera is through `events`.
   */
  attachViewport: (window: string, events: Channel<RenderViewportEvent>) =>
    __TAURI_INVOKE<RenderViewportId>("plugin:render|attach_viewport", { window, events }),
  /** Ask a viewport's camera to reset or dolly. */
  commandCamera: (viewport: RenderViewportId, command: RenderCameraCommand) =>
    __TAURI_INVOKE<void>("plugin:render|command_camera", { viewport, command }),
  /** Apply settings every native viewport draws with. */
  configure: (settings: RenderSettings) => __TAURI_INVOKE<void>("plugin:render|configure", { settings }),
  /** Say what a viewport's level could not draw: drawables the packer left out, sectors and spawned models unread. */
  describeProblems: (viewport: RenderViewportId) =>
    __TAURI_INVOKE<RenderLevelProblems>("plugin:render|describe_problems", { viewport }),
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
    __TAURI_INVOKE<Array<RenderSurfaceGeometry>>("plugin:render|measure_surfaces", { viewport }),
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
  poseModel: (viewport: RenderViewportId, pose: RenderModelPose) =>
    __TAURI_INVOKE<void>("plugin:render|pose_model", { viewport, pose }),
  /** Play a weather in a viewport's level from now on: a cycle by name, a keyframe set by hand, or nothing. */
  playWeather: (viewport: RenderViewportId, play: RenderWeatherPlay, transition: ERenderWeatherTransition) =>
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
  sendInput: (viewport: RenderViewportId, event: RenderInputEvent) =>
    __TAURI_INVOKE<void>("plugin:render|send_input", { viewport, event }),
  /** Describe a viewport's camera; described again from the same start, it keeps where it has been moved. */
  setCamera: (viewport: RenderViewportId, camera: RenderCamera) =>
    __TAURI_INVOKE<void>("plugin:render|set_camera", { viewport, camera }),
  /** Set the helpers drawn over one viewport's frame. */
  setOverlays: (viewport: RenderViewportId, overlays: Array<RenderOverlay>) =>
    __TAURI_INVOKE<void>("plugin:render|set_overlays", { viewport, overlays }),
  /** Set what one viewport draws its scene with. */
  setViewOptions: (viewport: RenderViewportId, options: RenderViewOptions) =>
    __TAURI_INVOKE<void>("plugin:render|set_view_options", { viewport, options }),
  /** Place a viewport where its element now is, and say what the page shows around it. */
  setViewportLayout: (viewport: RenderViewportId, layout: RenderViewportLayout) =>
    __TAURI_INVOKE<void>("plugin:render|set_viewport_layout", { viewport, layout }),
  /** Set how a viewport's weather clock runs. */
  setWeatherControl: (viewport: RenderViewportId, control: RenderWeatherControl) =>
    __TAURI_INVOKE<void>("plugin:render|set_weather_control", { viewport, control }),
  /** Draw the open level in a viewport, its sectors and textures read by the renderer; no session draws none. */
  showLevel: (viewport: RenderViewportId, sessionId: string | null) =>
    __TAURI_INVOKE<null>("plugin:render|show_level", { viewport, sessionId }),
  /**
   * Draw the open model in a viewport, `detail` down its collapse chain, its textures read by the renderer; no session
   * draws none.
   */
  showModel: (viewport: RenderViewportId, sessionId: string | null, detail: number | null) =>
    __TAURI_INVOKE<null>("plugin:render|show_model", { viewport, sessionId, detail }),
};
