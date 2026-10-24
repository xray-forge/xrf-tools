import { DialogFilter } from "@tauri-apps/plugin-dialog";
import { levelsCommands } from "@xrf/ipc/commands/levels";
import { EXrayExtension } from "@xrf/ipc/types/xrf-extension";

import {
  ILevelViewerSnapshot,
  parseLevelViewerSnapshot,
  serializeLevelViewerSnapshot,
} from "@/applications/level-viewer/lib/snapshot/level-viewer-snapshot";
import { withSupportedExtension } from "@/core/path/extension";

/** What the file dialogs offer a level view's snapshot as. */
export const LEVEL_VIEWER_SNAPSHOT_FILTERS: Array<DialogFilter> = [
  { extensions: [EXrayExtension.JSON], name: "Level view" },
];

/**
 * Writes a snapshot where a save dialog answered, its extension added where the name was typed without it.
 *
 * @param path - Where the save dialog answered.
 * @param snapshot - The view.
 * @returns Where it was written.
 */
export async function writeLevelViewerSnapshot(path: string, snapshot: ILevelViewerSnapshot): Promise<string> {
  const written: string = withSupportedExtension(path, [EXrayExtension.JSON], EXrayExtension.JSON);

  await levelsCommands.saveView(written, serializeLevelViewerSnapshot(snapshot));

  return written;
}

/**
 * @param path - A snapshot's file.
 * @returns The view it holds.
 * @throws {Error} Where the file cannot be read or is not a level view this build reads.
 */
export async function readLevelViewerSnapshot(path: string): Promise<ILevelViewerSnapshot> {
  return parseLevelViewerSnapshot(await levelsCommands.readView(path));
}
