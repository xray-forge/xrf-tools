// Auto-generated rust bindings. Do not edit it manually.

import { invoke as __TAURI_INVOKE } from "@/core/ipc/invoke";
import {
  AssetSource,
  SelectedVisualDescription,
  SessionId,
  SessionRestore,
  SessionSnapshot,
} from "@/core/ipc/types/xrf-app";
import { RenderViewportId } from "@/core/ipc/types/xrf-renderer";
import { XrayRoots } from "@/core/ipc/types/xrf-vfs";
import { VisualMotionBake } from "@/core/ipc/types/xrf-visual";

/** Commands */
export const visualsCommands = {
  /** Close the browses `session_ids` name, committed or opening. */
  closeBrowse: (sessionIds: Array<SessionId>) => __TAURI_INVOKE<void>("plugin:visuals|close_browse", { sessionIds }),
  /** Close the models `session_ids` name, committed or opening; a closed one answers no read. */
  closeModel: (sessionIds: Array<SessionId>) => __TAURI_INVOKE<void>("plugin:visuals|close_model", { sessionIds }),
  /** Close the posed motions `session_ids` name, committed or opening. */
  closeMotion: (sessionIds: Array<SessionId>) => __TAURI_INVOKE<void>("plugin:visuals|close_motion", { sessionIds }),
  /**
   * Describe one of the open visual's motions as a pose of it would hold it, its frames, bones and length, posing none;
   * read on the pool.
   */
  describeMotion: (sessionId: SessionId, name: string) =>
    __TAURI_INVOKE<VisualMotionBake>("plugin:visuals|describe_motion", { sessionId, name }),
  /**
   * Every motion the open visual can play, by name, read on the pool.
   *
   * Asked for rather than returned by `open_model`, because naming them means reading each animation file the visual
   * references - about fifty milliseconds each against a seventy millisecond open. The viewer already knows whether a
   * visual animates at all, from its references, so nothing needs this until something is about to play one.
   */
  listMotions: (sessionId: SessionId) => __TAURI_INVOKE<Array<string>>("plugin:visuals|list_motions", { sessionId }),
  /** Remember the browsed roots under an identity independent from the selected model. */
  openBrowse: (sessionId: SessionId, roots: XrayRoots) =>
    __TAURI_INVOKE<SessionSnapshot<XrayRoots>>("plugin:visuals|open_browse", { sessionId, roots }),
  /** Select a visual and return what it contains, with every reference it declares resolved, read on the pool. */
  openModel: (sessionId: SessionId, source: AssetSource, roots: XrayRoots) =>
    __TAURI_INVOKE<SessionSnapshot<SelectedVisualDescription>>("plugin:visuals|open_model", {
      sessionId,
      source,
      roots,
    }),
  /**
   * Pose the open visual through one of its motions, held as `motion_id`, an opening of the viewer's own, and report
   * what came out; baked on the pool.
   */
  openMotion: (sessionId: SessionId, motionId: SessionId, name: string) =>
    __TAURI_INVOKE<SessionSnapshot<VisualMotionBake>>("plugin:visuals|open_motion", { sessionId, motionId, name }),
  /** The roots `tool` browses in the asking window, the newest it opened. */
  restoreBrowse: (tool: string) => __TAURI_INVOKE<SessionRestore<XrayRoots>>("plugin:visuals|restore_browse", { tool }),
  /** The model `tool` holds in the asking window, the newest it opened, with its exact geometry identity. */
  restoreModel: (tool: string) =>
    __TAURI_INVOKE<SessionRestore<SelectedVisualDescription>>("plugin:visuals|restore_model", { tool }),
  /**
   * Draw the open model in a viewport, `detail` down its collapse chain, its textures read by the renderer; no session
   * draws none. Answers the scene drawn, which the viewport's load reports name, or none. Off the main thread, as every
   * show is, its ticket taken first so a model asked for later is never replaced by this one.
   */
  showModel: (viewport: RenderViewportId, sessionId: string | null, detail: number | null) =>
    __TAURI_INVOKE<number | null>("plugin:visuals|show_model", { viewport, sessionId, detail }),
};
