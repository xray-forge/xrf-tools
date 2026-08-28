import * as path from "@tauri-apps/api/path";
import { exists } from "@tauri-apps/plugin-fs";

import { systemCommands } from "@/core/bindings/commands/system";
import { EApplicationId } from "@/core/routing/application";
import { EPathRole } from "@/core/settings/lib/path/path-role";
import { EWorkspacePath, TWorkspacePaths } from "@/core/settings/lib/workspace-path";
import { Nullable, Optional } from "@/lib/types/general";

/** Where a role sits relative to the configured path that answered it. */
type TPathNarrowing = ReadonlyArray<string> | "parent";

/** One place a role reads from, and what it narrows to once that path is set. */
interface IPathRoleSource {
  from: EWorkspacePath;
  /** Segments beneath it, or the directory above it. Omitted when the configured path is the answer. */
  at?: TPathNarrowing;
}

/**
 * Where every role reads from, most specific first.
 */
const ROLE_CHAINS: Record<EPathRole, ReadonlyArray<IPathRoleSource>> = {
  [EPathRole.CONFIGS]: [{ from: EWorkspacePath.CONFIGS }, { from: EWorkspacePath.GAMEDATA, at: ["configs"] }],
  [EPathRole.SYSTEM_LTX]: [
    { from: EWorkspacePath.CONFIGS, at: ["system.ltx"] },
    { from: EWorkspacePath.GAMEDATA, at: ["configs", "system.ltx"] },
  ],
  [EPathRole.CONTENT_ROOT]: [{ from: EWorkspacePath.CONFIGS, at: "parent" }, { from: EWorkspacePath.GAMEDATA }],
  [EPathRole.TRANSLATIONS]: [
    { from: EWorkspacePath.TRANSLATIONS },
    { from: EWorkspacePath.GAMEDATA, at: ["configs", "text"] },
  ],
  [EPathRole.ALL_SPAWN]: [{ from: EWorkspacePath.GAMEDATA, at: ["spawns", "all.spawn"] }],
  [EPathRole.EQUIPMENT_SPRITE]: [{ from: EWorkspacePath.GAMEDATA, at: ["textures", "ui", "ui_icon_equipment.dds"] }],
  [EPathRole.EQUIPMENT_ICON_SOURCES]: [{ from: EWorkspacePath.ICON_SOURCES }],
  // Archive volumes are what an installation has and a loose tree has not, so there is nothing to fall back to.
  [EPathRole.ARCHIVES]: [{ from: EWorkspacePath.GAME_INSTALLATION }],
  [EPathRole.GAMEDATA]: [{ from: EWorkspacePath.GAMEDATA }],
  [EPathRole.BUILT_TRANSLATIONS]: [{ from: EWorkspacePath.GAME_INSTALLATION }, { from: EWorkspacePath.GAMEDATA }],
  // Game data first: a tree being edited is the likelier subject, and the installation behind it holds the rest.
  [EPathRole.VISUALS]: [{ from: EWorkspacePath.GAMEDATA }, { from: EWorkspacePath.GAME_INSTALLATION }],
};

/**
 * The path a role suggests, or `null` when nothing configured can answer it.
 *
 * @param role - What the field is asking for.
 * @param paths - The configured paths to derive from.
 * @returns The suggested path, or `null`.
 */
export async function resolvePathRole(role: EPathRole, paths: TWorkspacePaths): Promise<Nullable<string>> {
  for (const source of ROLE_CHAINS[role]) {
    const configured: Nullable<string> = paths[source.from];

    if (configured) {
      return narrow(configured, source.at);
    }
  }

  return null;
}

/**
 * The path a role suggests, but only while it is there.
 *
 * For fields whose subject either exists or is not worth suggesting - a built spawn file, an installation to unpack -
 * so an absent one leaves the field empty rather than showing a path with an error under it.
 *
 * @param role - What the field is asking for.
 * @param paths - The configured paths to derive from.
 * @returns The suggested path when it exists, otherwise `null`.
 */
export async function resolveExistingPathRole(role: EPathRole, paths: TWorkspacePaths): Promise<Nullable<string>> {
  const resolved: Nullable<string> = await resolvePathRole(role, paths);

  if (!resolved) {
    return null;
  }

  return (await exists(resolved)) ? resolved : null;
}

/**
 * Every configured tree an asset reference may resolve in, in search order.
 *
 * Game data first, then the installation behind it, which is how a mod is layered: a loose tree carries only what it
 * changed, and the game it was built against still answers for the rest. Unset entries travel through, because
 * `createRoots` drops them.
 *
 * @param paths - The configured paths to derive from.
 * @returns The configured roots, highest priority first.
 */
export function configuredAssetRoots(paths: TWorkspacePaths): Array<Nullable<string>> {
  return [paths[EWorkspacePath.GAMEDATA], paths[EWorkspacePath.GAME_INSTALLATION]];
}

/**
 * Where one application writes its results.
 *
 * Named for the application rather than for the kind of result, so every tool that writes gets a place of its own
 * without a table anyone has to extend. Falls back to a directory the application chooses for itself, which is beside
 * the executable when that is writable and in its data directory when it is not.
 *
 * @param application - Application whose output directory is wanted.
 * @param paths - The configured paths to derive from.
 * @param fileName - File inside that directory, when the field names a file rather than a directory.
 * @returns The suggested output path, or `null` when no root could be resolved.
 */
export async function resolveOutputPath(
  application: EApplicationId,
  paths: TWorkspacePaths,
  fileName?: string
): Promise<Nullable<string>> {
  const root: Nullable<string> = paths[EWorkspacePath.OUTPUT] ?? (await getDefaultOutputRoot());

  if (!root) {
    return null;
  }

  return fileName ? path.resolve(root, application, fileName) : path.resolve(root, application);
}

/** Narrows the configured path that answered a role to what the role actually asks for. */
function narrow(configured: string, at: Optional<TPathNarrowing>): Promise<string> | string {
  if (!at) {
    return configured;
  }

  return at === "parent" ? path.dirname(configured) : path.resolve(configured, ...at);
}

/**
 * The directory the application writes into when no output path is configured.
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
