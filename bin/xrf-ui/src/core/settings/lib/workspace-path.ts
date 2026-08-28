import { EPathRole } from "@/core/settings/lib/path/path-role";
import { Nullable, Optional } from "@/lib/types/general";

/**
 * A path the application remembers for every tool at once.
 *
 * Game data is the one most users have; the rest are overrides for a layout that puts something somewhere else, and an
 * unset override simply means the value derives from game data instead. Nothing here describes a repository layout: a
 * source tree is expressed by pointing the individual overrides at it.
 */
export enum EWorkspacePath {
  /** Loose game data tree, and the root every other path derives from when it is not set explicitly. */
  GAMEDATA = "gamedata",
  /** Installed game, declaring its own layout with `fsgame.ltx`. */
  GAME_INSTALLATION = "gameInstallation",
  /** Directory of LTX configuration files. */
  CONFIGS = "configs",
  /** Directory of translation sources. */
  TRANSLATIONS = "translations",
  /** Directory of individual equipment icon images. */
  ICON_SOURCES = "iconSources",
  /** Parent directory everything the tools write goes under. */
  OUTPUT = "output",
}

export type TWorkspacePaths = Record<EWorkspacePath, Nullable<string>>;

/** Everything about one configured path: what it is called, where it is kept, and how a row reports on it. */
export interface IWorkspacePathDescriptor {
  id: EWorkspacePath;
  label: string;
  description: string;
  /** Title of the dialog that chooses it. */
  title: string;
  /** Where the choice is remembered. */
  storageKey: string;
  /**
   * Names something the virtual file system mounts, rather than a plain directory.
   *
   * A root is described by planning it, because whether a directory holds anything an engine would load is not a
   * question the frontend can answer. Everything else is only checked for being there.
   */
  isRoot?: boolean;
  /**
   * Role whose derived value stands in while nothing is set.
   *
   * Absent where nothing derives one, which is how a row says that leaving it empty means no suggestion at all rather
   * than a suggestion taken from somewhere else.
   */
  derivedFrom?: EPathRole;
}

/**
 * Every configured path, in the order the settings screen lists them.
 */
export const WORKSPACE_PATHS: ReadonlyArray<IWorkspacePathDescriptor> = [
  {
    id: EWorkspacePath.GAMEDATA,
    label: "Gamedata",
    description: "Loose game data tree. Every tool starts from here unless something below says otherwise.",
    title: "Select gamedata directory",
    storageKey: "xrf-gamedata-path",
    isRoot: true,
  },
  {
    id: EWorkspacePath.GAME_INSTALLATION,
    label: "Game installation",
    description: "Installed game declaring `fsgame.ltx`. Archived assets are read from here.",
    title: "Select game installation directory",
    storageKey: "xrf-game-installation-path",
    isRoot: true,
  },
  {
    id: EWorkspacePath.CONFIGS,
    label: "Configs",
    description: "Directory of LTX configuration files.",
    title: "Select configs directory",
    storageKey: "xrf-configs-path",
    derivedFrom: EPathRole.CONFIGS,
  },
  {
    id: EWorkspacePath.TRANSLATIONS,
    label: "Translations",
    description: "Directory of translation sources.",
    title: "Select translations directory",
    storageKey: "xrf-translations-path",
    derivedFrom: EPathRole.TRANSLATIONS,
  },
  {
    id: EWorkspacePath.ICON_SOURCES,
    label: "Icon sources",
    description: "Directory of individual equipment icon images the packer composes a sprite from.",
    title: "Select icon sources directory",
    storageKey: "xrf-icon-sources-path",
  },
  {
    id: EWorkspacePath.OUTPUT,
    label: "Output",
    description: "Where tools write their results. Each one gets a directory of its own beneath it.",
    title: "Select output directory",
    storageKey: "xrf-output-path",
  },
];

/** The path most setups need, which every override is an exception to. */
export const PRIMARY_WORKSPACE_PATH: IWorkspacePathDescriptor = WORKSPACE_PATHS[0];

/** The overrides, which is every path except the primary one. */
export const WORKSPACE_PATH_OVERRIDES: ReadonlyArray<IWorkspacePathDescriptor> = WORKSPACE_PATHS.slice(1);

/** Nothing configured, which is also what the application starts from. */
export function createEmptyWorkspacePaths(): TWorkspacePaths {
  // Built from the table rather than written out again, so the two cannot disagree about which paths exist. The cast
  // states what the loop cannot: every key of the record is covered, which the table's own test proves.
  const paths: Partial<TWorkspacePaths> = {};

  for (const { id } of WORKSPACE_PATHS) {
    paths[id] = null;
  }

  return paths as TWorkspacePaths;
}

/**
 * @param id - Path to describe.
 * @returns How that path is named, kept and reported on, or `undefined` when nothing describes it.
 */
export function getWorkspacePath(id: EWorkspacePath): Optional<IWorkspacePathDescriptor> {
  return WORKSPACE_PATHS.find((it: IWorkspacePathDescriptor) => it.id === id);
}
