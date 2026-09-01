import { ContainerConfig } from "@wirestate/core";
import { ComponentType, ReactElement } from "react";

/** Canonical application identity shared by its directory, route, descriptor, and component stem. */
export enum EApplicationId {
  ARCHIVES_EXPLORER = "archives-explorer",
  ARCHIVES_PACKER = "archives-packer",
  ARCHIVES_UNPACKER = "archives-unpacker",
  CHARACTERS_EXPLORER = "characters-explorer",
  CONFIGS_EXPLORER = "configs-explorer",
  CONFIGS_FORMATTER = "configs-formatter",
  CONFIGS_VERIFIER = "configs-verifier",
  DIALOGS_EDITOR = "dialogs-editor",
  GAMEDATA_VERIFIER = "gamedata-verifier",
  EXPORTS_EXPLORER = "exports-explorer",
  INFO_PORTIONS_EXPLORER = "info-portions-explorer",
  SPAWN_EDITOR = "spawn-editor",
  SPAWN_PACKER = "spawn-packer",
  SPAWN_UNPACKER = "spawn-unpacker",
  SPRITE_DESCRIPTION_EDITOR = "sprite-description-editor",
  SPRITE_DESCRIPTION_PACKER = "sprite-description-packer",
  SPRITE_DESCRIPTION_UNPACKER = "sprite-description-unpacker",
  SPRITE_EQUIPMENT_EDITOR = "sprite-equipment-editor",
  SPRITE_EQUIPMENT_PACKER = "sprite-equipment-packer",
  SPRITE_EQUIPMENT_UNPACKER = "sprite-equipment-unpacker",
  TASKS_EXPLORER = "tasks-explorer",
  TRANSLATIONS_BUILDER = "translations-builder",
  TRANSLATIONS_EDITOR = "translations-editor",
  TRANSLATIONS_FORMATTER = "translations-formatter",
  TRANSLATIONS_PARSER = "translations-parser",
  TRANSLATIONS_VERIFIER = "translations-verifier",
  VISUALS_EXPLORER = "visuals-explorer",
  VISUALS_SEQUENCER = "visuals-sequencer",
}

/**
 * The family an application belongs to.
 */
export enum EApplicationGroupId {
  ARCHIVES = "archives",
  CONFIGS = "configs",
  DIALOGS = "dialogs",
  EXPORTS = "exports",
  GAMEDATA = "gamedata",
  GAMEPLAY = "gameplay",
  SPAWNS = "spawns",
  SPRITES = "sprites",
  TRANSLATIONS = "translations",
  VISUALS = "visuals",
}

/**
 * Whether an application does anything yet.
 *
 * `PLANNED` surfaces exist as signposts on the home page: the roster is the roadmap, so an unbuilt
 * screen is visible but inert rather than silently missing. Developer mode opens them anyway.
 */
export enum EApplicationStatus {
  PLANNED = "planned",
  READY = "ready",
}

/**
 * What an application tells its user about itself, beyond the launcher one-liner.
 *
 * The shape is the rubric: nuances and limitations over a walkthrough of controls the screen already
 * shows. Backticked spans render as code; that is the only markup any string carries.
 */
export interface IApplicationHelp {
  /** What the tool is for and when to reach for it, in a few sentences. */
  summary: string;
  /** The typical run, as ordered steps. */
  workflow?: ReadonlyArray<string>;
  /** Behaviors worth knowing that the screen does not make obvious. */
  nuances?: ReadonlyArray<string>;
  /** What the tool deliberately does not do, and its known constraints. */
  limitations?: ReadonlyArray<string>;
  /** Applications belonging to the same workflow. */
  relatedTools?: ReadonlyArray<EApplicationId>;
}

export interface IApplicationDescriptor {
  id: EApplicationId;
  group: EApplicationGroupId;
  /** The one name this application answers to, everywhere. */
  label: string;
  description: string;
  icon: ReactElement;
  path: string;
  status: EApplicationStatus;
  /** In-application help. Required for `READY` applications once the roster is covered. */
  help?: IApplicationHelp;
  /** The container this application's services live in. Omit it to run in the root one. */
  container?: Omit<ContainerConfig, "parent">;
  Component: ComponentType;
  /** Pulls this application's chunk in before it is navigated to. */
  preload?: () => Promise<unknown>;
}

export interface IApplicationGroupAccent {
  light: string;
  dark: string;
}

export interface IApplicationGroup {
  id: EApplicationGroupId;
  label: string;
  icon: ReactElement;
  accent: IApplicationGroupAccent;
}

/** Sources that raise notifications without owning an application of their own. */
export const APPLICATION_SOURCE: string = "application";
