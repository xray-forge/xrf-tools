import { join } from "@tauri-apps/api/path";
import { DialogFilter } from "@tauri-apps/plugin-dialog";
import { mkdir, readTextFile, writeTextFile } from "@tauri-apps/plugin-fs";
import { getPathName } from "@xrf/path";
import { Nullable } from "@xrf/types";

import {
  ILevelViewerSnapshot,
  parseLevelViewerSnapshot,
  serializeLevelViewerSnapshot,
} from "@/applications/level-viewer/lib/level-viewer-snapshot";
import { systemCommands } from "@/core/ipc/commands/system";
import { EXrayExtension } from "@/core/ipc/types/xrf-extension";
import { withSupportedExtension } from "@/core/path/extension";

/** What the file dialogs offer a level view's snapshot as. */
export const LEVEL_VIEWER_SNAPSHOT_FILTERS: Array<DialogFilter> = [
  { extensions: [EXrayExtension.JSON], name: "Level view" },
];

/** Where the level viewer keeps its views within the application's own data: `level-viewer/views`. */
const LEVEL_VIEWER_SNAPSHOT_FOLDER: ReadonlyArray<string> = ["level-viewer", "views"];

/**
 * The folder the file dialogs open on, made where it is not yet: under the application's own data beside its
 * executable, where its logs are, named for the tool, so the owner and the agents driving the viewer share it.
 *
 * @returns The folder's path.
 */
export async function getLevelViewerSnapshotFolder(): Promise<string> {
  const folder: string = await join(await systemCommands.getAppDataRoot(), ...LEVEL_VIEWER_SNAPSHOT_FOLDER);

  await mkdir(folder, { recursive: true });

  return folder;
}

/**
 * @param level - The level the view is of, as a snapshot names it, or null for none.
 * @param at - When it is taken.
 * @returns A file name naming the level and the moment, which sorts by time within a level.
 */
export function toLevelViewerSnapshotName(level: Nullable<string>, at: Date): string {
  const date: string = `${at.getFullYear()}${pad(at.getMonth() + 1)}${pad(at.getDate())}`;
  const time: string = `${pad(at.getHours())}${pad(at.getMinutes())}${pad(at.getSeconds())}`;

  return `${level ? getPathName(level) : "view"}-${date}-${time}.${EXrayExtension.JSON}`;
}

/**
 * Writes a snapshot where a save dialog answered, its extension added where the name was typed without it.
 *
 * @param path - Where the save dialog answered.
 * @param snapshot - The view.
 * @returns Where it was written.
 */
export async function writeLevelViewerSnapshot(path: string, snapshot: ILevelViewerSnapshot): Promise<string> {
  const written: string = withSupportedExtension(path, [EXrayExtension.JSON], EXrayExtension.JSON);

  await writeTextFile(written, serializeLevelViewerSnapshot(snapshot));

  return written;
}

/**
 * @param path - A snapshot's file.
 * @returns The view it holds.
 * @throws {Error} Where the file cannot be read or is not a level view this build reads.
 */
export async function readLevelViewerSnapshot(path: string): Promise<ILevelViewerSnapshot> {
  return parseLevelViewerSnapshot(await readTextFile(path));
}

function pad(value: number): string {
  return String(value).padStart(2, "0");
}
