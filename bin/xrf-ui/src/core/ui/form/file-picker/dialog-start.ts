import { exists } from "@tauri-apps/plugin-fs";

import { getPathDirectory } from "@/lib/path/separator";
import { Nullable, Optional } from "@/lib/types/general";

export interface IDialogStartOptions {
  /**
   * Whether the field names something to write.
   *
   * A destination need not exist, so its own path is still worth offering back once the directory around it does:
   * falling back to that directory would discard the name chosen for the file.
   */
  isSave?: boolean;
}

/**
 * Where a native dialog should open for a field that already holds a path.
 *
 * Answering nothing leaves it to the host, which means wherever a dialog was last used - in any application, for any
 * purpose - so a field plainly showing one game data tree would open in another. The path on screen is a better guess
 * than that whenever it leads anywhere: the path itself when it is there, otherwise the directory above it, which is
 * where something moved or renamed was last seen.
 *
 * Separate from `usePathState` because deciding this reads the filesystem, and that hook's whole interface is calling
 * the dialog. Separate from `usePathField` because it is a rule about paths rather than about a field's state.
 *
 * @param current - The path the field is holding.
 * @param options - How the field uses its path.
 * @param options.isSave - Whether the field names something to write.
 * @returns The path to open at, or nothing to leave it to the host.
 */
export async function resolveDialogStart(
  current: Nullable<string>,
  { isSave = false }: IDialogStartOptions = {}
): Promise<Optional<string>> {
  if (!current) {
    return undefined;
  }

  if (await isPresent(current)) {
    return current;
  }

  const directory: string = getPathDirectory(current);

  if (!directory || !(await isPresent(directory))) {
    return undefined;
  }

  return isSave ? current : directory;
}

/** Whether a path is there, treating a refused look as absent: this only decides where a dialog opens. */
async function isPresent(path: string): Promise<boolean> {
  return exists(path).catch(() => false);
}
