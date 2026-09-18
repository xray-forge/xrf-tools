// Auto-generated rust bindings. Do not edit it manually.

import { invoke as __TAURI_INVOKE } from "@/core/ipc/invoke";
import {
  LevelEntry,
  LevelSource,
  SelectedLevelDescription,
  SessionId,
  SessionRestore,
  SessionSnapshot,
} from "@/core/ipc/types/xrf-app";
import { XrayRoots } from "@/core/ipc/types/xrf-vfs";
import { SectorDescription } from "@/core/ipc/types/xrf-visual";

/** Commands */
export const levelsCommands = {
  /** Release only the openings owned by the departing viewer. */
  closeLevel: (sessionIds: Array<SessionId>) => __TAURI_INVOKE<null>("plugin:levels|close_level", { sessionIds }),
  /** Restore the committed level descriptor without reading the level again. */
  getLevel: () => __TAURI_INVOKE<SessionRestore<SelectedLevelDescription>>("plugin:levels|get_level"),
  /** Every compiled level the mounted roots hold, loose or archived alike. */
  listLevels: (roots: XrayRoots) => __TAURI_INVOKE<Array<LevelEntry>>("plugin:levels|list_levels", { roots }),
  /** Select a compiled level and report what it is built out of, without reading any of its geometry. */
  openLevel: (sessionId: SessionId, source: LevelSource, roots: XrayRoots) =>
    __TAURI_INVOKE<SessionSnapshot<SelectedLevelDescription>>("plugin:levels|open_level", { sessionId, source, roots }),
  /**
   * Pack one sector of the open level and report what it became.
   *
   * The bytes stay here until `read_sector` takes them, so the description and the buffer it describes are the one
   * pack rather than two.
   */
  openSector: (sessionId: SessionId, sectorId: SessionId, sector: number) =>
    __TAURI_INVOKE<SessionSnapshot<SectorDescription>>("plugin:levels|open_sector", { sessionId, sectorId, sector }),
};
