// Auto-generated rust bindings. Do not edit it manually.

import { Channel } from "@tauri-apps/api/core";

import { invoke as __TAURI_INVOKE } from "@/core/ipc/invoke";
import {
  RenderCamera,
  RenderCameraCommand,
  RenderInputEvent,
  RenderSettings,
  RenderViewportEvent,
  RenderViewportId,
  RenderViewportLayout,
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
  /** Stop drawing a viewport; the GPU goes a few seconds after the last one. */
  detachViewport: (viewport: RenderViewportId) => __TAURI_INVOKE<void>("plugin:render|detach_viewport", { viewport }),
  /** Hand a viewport one gesture the page heard over it. */
  sendInput: (viewport: RenderViewportId, event: RenderInputEvent) =>
    __TAURI_INVOKE<void>("plugin:render|send_input", { viewport, event }),
  /** Describe a viewport's camera; described again from the same start, it keeps where it has been moved. */
  setCamera: (viewport: RenderViewportId, camera: RenderCamera) =>
    __TAURI_INVOKE<void>("plugin:render|set_camera", { viewport, camera }),
  /** Place a viewport where its element now is, and say what the page shows around it. */
  setViewportLayout: (viewport: RenderViewportId, layout: RenderViewportLayout) =>
    __TAURI_INVOKE<void>("plugin:render|set_viewport_layout", { viewport, layout }),
};
