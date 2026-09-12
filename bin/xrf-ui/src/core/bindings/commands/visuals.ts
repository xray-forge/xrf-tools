// Auto-generated rust bindings. Do not edit it manually.

import { invoke as __TAURI_INVOKE } from "@tauri-apps/api/core";

import {
  SelectedVisualDescription,
  SessionId,
  SessionRestore,
  SessionSnapshot,
  VisualSource,
} from "@/core/bindings/types/xrf-app";
import { XrayRoots } from "@/core/bindings/types/xrf-vfs";
import { VisualMotionBake } from "@/core/bindings/types/xrf-visual";

/** Commands */
export const visualsCommands = {
  /** Release only the openings owned by the departing viewer. */
  closeBrowse: (sessionIds: Array<SessionId>) => __TAURI_INVOKE<null>("plugin:visuals|close_browse", { sessionIds }),
  /** Release only the openings owned by the departing viewer. */
  closeModel: (sessionIds: Array<SessionId>) => __TAURI_INVOKE<null>("plugin:visuals|close_model", { sessionIds }),
  /** Restore the committed browse scope. */
  getBrowse: () => __TAURI_INVOKE<SessionRestore<XrayRoots>>("plugin:visuals|get_browse"),
  /** Restore the committed model descriptor and its exact geometry identity. */
  getModel: () => __TAURI_INVOKE<SessionRestore<SelectedVisualDescription>>("plugin:visuals|get_model"),
  /**
   * Every motion the open visual can play, by name.
   *
   * Asked for rather than returned by `open_model`, because naming them means reading each animation file the visual
   * references - about fifty milliseconds each against a seventy millisecond open. The viewer already knows whether a
   * visual animates at all, from its references, so nothing needs this until something is about to play one.
   */
  listMotions: (sessionId: SessionId) => __TAURI_INVOKE<Array<string>>("plugin:visuals|list_motions", { sessionId }),
  /** Remember the browsed roots with an identity independent from the selected model. */
  openBrowse: (sessionId: SessionId, roots: XrayRoots) =>
    __TAURI_INVOKE<SessionSnapshot<XrayRoots>>("plugin:visuals|open_browse", { sessionId, roots }),
  /**
   * Select a visual and return what it contains, with every reference it declares resolved.
   *
   * Geometry is packed here and parked, so the `read_geometry` that follows serves the same parse rather than repeating
   * it. The bytes are not returned: a typed command cannot carry them, which is why they are read separately.
   *
   * Resolution happens once, for the whole dependency set, in this one call. That is what keeps a model with forty
   * textures from costing forty round trips, and it is why the outcomes travel with the description rather than being
   * asked for afterwards.
   */
  openModel: (sessionId: SessionId, source: VisualSource, roots: XrayRoots) =>
    __TAURI_INVOKE<SessionSnapshot<SelectedVisualDescription>>("plugin:visuals|open_model", {
      sessionId,
      source,
      roots,
    }),
  /**
   * Pose the open visual through one of its motions, and report what came out.
   *
   * Every frame is baked here and parked, so the `read_motion` that follows serves the same pose rather than composing
   * it again - the same split geometry uses, and for the same reason: a typed command cannot carry the bytes.
   *
   * Baked whole rather than sampled per frame because playback runs at thirty frames a second. A measured motion
   * averages 78 frames, which for a fifty bone skeleton is tens of kilobytes: cheaper once than eighty times.
   */
  openMotion: (sessionId: SessionId, motionId: SessionId, name: string) =>
    __TAURI_INVOKE<SessionSnapshot<VisualMotionBake>>("plugin:visuals|open_motion", { sessionId, motionId, name }),
};
