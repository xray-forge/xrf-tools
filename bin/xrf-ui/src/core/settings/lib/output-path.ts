import * as path from "@tauri-apps/api/path";

import { systemCommands } from "@/core/bindings/commands/system";
import { EApplicationId } from "@/core/routing/application";
import { Nullable } from "@/lib/types/general";

/**
 * Where one application writes its results.
 *
 * The one path this application still guesses, and the reason it may: it is a fact about the application rather than
 * about the person using it. Every other path is named where it is used, because only someone with the game data in
 * front of them knows which tree they mean.
 *
 * Named for the application rather than for the kind of result, so every tool that writes gets a place of its own
 * without a table anyone has to extend.
 *
 * @param application - Application whose output directory is wanted.
 * @param fileName - File inside that directory, when the field names a file rather than a directory.
 * @returns The suggested output path, or `null` when no root could be resolved.
 */
export async function resolveOutputPath(application: EApplicationId, fileName?: string): Promise<Nullable<string>> {
  const root: Nullable<string> = await getDefaultOutputRoot();

  if (!root) {
    return null;
  }

  return fileName ? path.resolve(root, application, fileName) : path.resolve(root, application);
}

/**
 * The directory the application writes into.
 *
 * Answered by the backend because only it can see where the executable lives and whether that directory accepts a
 * file. A failure is no suggestion rather than an error: this runs to fill a field nobody has filled in yet.
 */
async function getDefaultOutputRoot(): Promise<Nullable<string>> {
  try {
    return await systemCommands.getDefaultOutputRoot();
  } catch {
    return null;
  }
}
