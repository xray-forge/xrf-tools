// Auto-generated rust bindings. Do not edit it manually.

import { invoke as __TAURI_INVOKE } from "#/invoke";
import { IIpcCommandSignature } from "#/signature";
import { AssetSource, SessionId, SessionRestore, SessionSnapshot } from "#/types/xrf-app-core";
import { SelectedVisualDescription } from "#/types/xrf-app-visuals";
import { RenderViewportId } from "#/types/xrf-renderer";
import { XrayRoots } from "#/types/xrf-vfs";
import { VisualMotionBake } from "#/types/xrf-visual";

/** The wire name of every command of the `visuals` plugin. */
export enum EVisualsCommand {
  CLOSE_BROWSE = "plugin:visuals|close_browse",
  CLOSE_MODEL = "plugin:visuals|close_model",
  CLOSE_MOTION = "plugin:visuals|close_motion",
  DESCRIBE_MOTION = "plugin:visuals|describe_motion",
  LIST_MOTIONS = "plugin:visuals|list_motions",
  OPEN_BROWSE = "plugin:visuals|open_browse",
  OPEN_MODEL = "plugin:visuals|open_model",
  OPEN_MOTION = "plugin:visuals|open_motion",
  RESTORE_BROWSE = "plugin:visuals|restore_browse",
  RESTORE_MODEL = "plugin:visuals|restore_model",
  SHOW_MODEL = "plugin:visuals|show_model",
}

/** Commands */
export const visualsCommands = {
  /** Close the browses `session_ids` name, committed or opening. */
  closeBrowse: (sessionIds: Array<SessionId>) => __TAURI_INVOKE<void>(EVisualsCommand.CLOSE_BROWSE, { sessionIds }),
  /** Close the models `session_ids` name, committed or opening; a closed one answers no read. */
  closeModel: (sessionIds: Array<SessionId>) => __TAURI_INVOKE<void>(EVisualsCommand.CLOSE_MODEL, { sessionIds }),
  /** Close the posed motions `session_ids` name, committed or opening. */
  closeMotion: (sessionIds: Array<SessionId>) => __TAURI_INVOKE<void>(EVisualsCommand.CLOSE_MOTION, { sessionIds }),
  /**
   * Describe one of the open visual's motions as a pose of it would hold it, its frames, bones and length, posing none;
   * read on the pool.
   */
  describeMotion: (sessionId: SessionId, name: string) =>
    __TAURI_INVOKE<VisualMotionBake>(EVisualsCommand.DESCRIBE_MOTION, { sessionId, name }),
  /**
   * Every motion the open visual can play, by name, read on the pool.
   *
   * Asked for rather than returned by `open_model`, because naming them means reading each animation file the visual
   * references - about fifty milliseconds each against a seventy millisecond open. The viewer already knows whether a
   * visual animates at all, from its references, so nothing needs this until something is about to play one.
   */
  listMotions: (sessionId: SessionId) => __TAURI_INVOKE<Array<string>>(EVisualsCommand.LIST_MOTIONS, { sessionId }),
  /** Remember the browsed roots under an identity independent from the selected model. */
  openBrowse: (sessionId: SessionId, roots: XrayRoots) =>
    __TAURI_INVOKE<SessionSnapshot<XrayRoots>>(EVisualsCommand.OPEN_BROWSE, { sessionId, roots }),
  /** Select a visual and return what it contains, with every reference it declares resolved, read on the pool. */
  openModel: (sessionId: SessionId, source: AssetSource, roots: XrayRoots) =>
    __TAURI_INVOKE<SessionSnapshot<SelectedVisualDescription>>(EVisualsCommand.OPEN_MODEL, {
      sessionId,
      source,
      roots,
    }),
  /**
   * Pose the open visual through one of its motions, held as `motion_id`, an opening of the viewer's own, and report
   * what came out; baked on the pool.
   */
  openMotion: (sessionId: SessionId, motionId: SessionId, name: string) =>
    __TAURI_INVOKE<SessionSnapshot<VisualMotionBake>>(EVisualsCommand.OPEN_MOTION, { sessionId, motionId, name }),
  /** The roots `tool` browses in the asking window, the newest it opened. */
  restoreBrowse: (tool: string) => __TAURI_INVOKE<SessionRestore<XrayRoots>>(EVisualsCommand.RESTORE_BROWSE, { tool }),
  /** The model `tool` holds in the asking window, the newest it opened, with its exact geometry identity. */
  restoreModel: (tool: string) =>
    __TAURI_INVOKE<SessionRestore<SelectedVisualDescription>>(EVisualsCommand.RESTORE_MODEL, { tool }),
  /**
   * Draw the open model in a viewport, `detail` down its collapse chain, its textures read by the renderer; no session
   * draws none. Answers the scene drawn, which the viewport's load reports name, or none. Off the main thread, as every
   * show is, its ticket taken first so a model asked for later is never replaced by this one.
   */
  showModel: (viewport: RenderViewportId, sessionId: string | null, detail: number | null) =>
    __TAURI_INVOKE<number | null>(EVisualsCommand.SHOW_MODEL, { viewport, sessionId, detail }),
};

/** What each command of the `visuals` plugin takes and resolves to, keyed by its wire name. */
export interface IVisualsCommandSignatures {
  [EVisualsCommand.CLOSE_BROWSE]: IIpcCommandSignature<typeof visualsCommands.closeBrowse, ["sessionIds"]>;
  [EVisualsCommand.CLOSE_MODEL]: IIpcCommandSignature<typeof visualsCommands.closeModel, ["sessionIds"]>;
  [EVisualsCommand.CLOSE_MOTION]: IIpcCommandSignature<typeof visualsCommands.closeMotion, ["sessionIds"]>;
  [EVisualsCommand.DESCRIBE_MOTION]: IIpcCommandSignature<typeof visualsCommands.describeMotion, ["sessionId", "name"]>;
  [EVisualsCommand.LIST_MOTIONS]: IIpcCommandSignature<typeof visualsCommands.listMotions, ["sessionId"]>;
  [EVisualsCommand.OPEN_BROWSE]: IIpcCommandSignature<typeof visualsCommands.openBrowse, ["sessionId", "roots"]>;
  [EVisualsCommand.OPEN_MODEL]: IIpcCommandSignature<
    typeof visualsCommands.openModel,
    ["sessionId", "source", "roots"]
  >;
  [EVisualsCommand.OPEN_MOTION]: IIpcCommandSignature<
    typeof visualsCommands.openMotion,
    ["sessionId", "motionId", "name"]
  >;
  [EVisualsCommand.RESTORE_BROWSE]: IIpcCommandSignature<typeof visualsCommands.restoreBrowse, ["tool"]>;
  [EVisualsCommand.RESTORE_MODEL]: IIpcCommandSignature<typeof visualsCommands.restoreModel, ["tool"]>;
  [EVisualsCommand.SHOW_MODEL]: IIpcCommandSignature<
    typeof visualsCommands.showModel,
    ["viewport", "sessionId", "detail"]
  >;
}
