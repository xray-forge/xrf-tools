import { open, save } from "@tauri-apps/plugin-dialog";
import { exists } from "@tauri-apps/plugin-fs";
import { getPathDirectory } from "@xrf/path";
import { Nullable, Optional } from "@xrf/types";

import { IFileFilter } from "@/core/path/file-filters";

export interface INativeDialogOptions {
  title: string;
  /**
   * The path the dialog opens at or near: the path itself when it is there, otherwise the directory above it, and the
   * host's choice when neither is. A save dialog keeps a missing name whose directory is there, and a bare name too.
   */
  start?: Nullable<string>;
}

export interface INativeFileDialogOptions extends INativeDialogOptions {
  /** The file types offered, in order; every file when absent. */
  filters?: Nullable<ReadonlyArray<IFileFilter>>;
}

export interface INativeDirectoryDialogOptions extends INativeDialogOptions {
  /** Whether the directory is a destination that need not exist yet, so its own path is offered back. */
  isDestination?: boolean;
}

/**
 * Asks for one existing file to read.
 *
 * @returns The file picked, or null where the dialog was dismissed.
 */
export async function askToOpen({ title, start, filters }: INativeFileDialogOptions): Promise<Nullable<string>> {
  return open({
    defaultPath: await resolveDialogStart(start, { isSave: false }),
    directory: false,
    filters: toDialogFilters(filters),
    multiple: false,
    title,
  });
}

/**
 * Asks for one directory.
 *
 * @returns The directory picked, or null where the dialog was dismissed.
 */
export async function askToOpenDirectory({
  title,
  start,
  isDestination = false,
}: INativeDirectoryDialogOptions): Promise<Nullable<string>> {
  return open({
    defaultPath: await resolveDialogStart(start, { isSave: isDestination }),
    directory: true,
    multiple: false,
    title,
  });
}

/**
 * Asks where to write one file; the host asks about replacing one that is there.
 *
 * @returns The path chosen, as typed, or null where the dialog was dismissed.
 */
export async function askToSave({ title, start, filters }: INativeFileDialogOptions): Promise<Nullable<string>> {
  return save({
    defaultPath: await resolveDialogStart(start, { isSave: true }),
    filters: toDialogFilters(filters),
    title,
  });
}

/**
 * Where a native dialog opens for a path the caller holds.
 *
 * Left to the host, a dialog opens wherever one was last used, in any tool, so the path on screen is the better guess
 * whenever it leads anywhere. A destination's own path is offered once its directory exists, as falling back to the
 * directory would discard the name chosen for it.
 *
 * @returns The path to open at, or nothing to leave it to the host.
 */
async function resolveDialogStart(
  current: Optional<Nullable<string>>,
  { isSave }: { isSave: boolean }
): Promise<Optional<string>> {
  if (!current) {
    return undefined;
  }

  const directory: string = getPathDirectory(current);

  // A name alone suggests what to call the file, in whichever directory the host opens.
  if (isSave && !directory) {
    return current;
  }

  if (await isPresent(current)) {
    return current;
  }

  if (!directory || !(await isPresent(directory))) {
    return undefined;
  }

  return isSave ? current : directory;
}

/** Whether a path is there, treating a refused look as absent: this only decides where a dialog opens. */
async function isPresent(path: string): Promise<boolean> {
  return exists(path).catch(() => false);
}

/** The filters as the plugin takes them: a mutable list, or none for every file. */
function toDialogFilters(filters: Optional<Nullable<ReadonlyArray<IFileFilter>>>): Optional<Array<IFileFilter>> {
  return filters ? [...filters] : undefined;
}
