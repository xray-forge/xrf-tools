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
 * @param options - The dialog's options.
 * @param options.title - The dialog's title.
 * @param options.start - The path it opens at or near.
 * @param options.filters - The file types offered.
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
 * @param options - The dialog's options.
 * @param options.title - The dialog's title.
 * @param options.start - The path it opens at or near.
 * @param options.isDestination - Whether the directory may not exist yet.
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
 * @param options - The dialog's options.
 * @param options.title - The dialog's title.
 * @param options.start - The path it opens at or near; a bare name is offered as the file's name.
 * @param options.filters - The file types offered.
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
 * Answering nothing leaves it to the host, which means wherever a dialog was last used - in any tool, for any purpose -
 * so a field plainly showing one game data tree would open in another. The path on screen is a better guess whenever it
 * leads anywhere. A destination need not exist, so its own path is offered back once the directory around it does:
 * falling back to that directory would discard the name chosen for it.
 *
 * @param current - The path held.
 * @param options - How the path is used.
 * @param options.isSave - Whether it names something to write.
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
