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
  DESCRIPTION_ICONS_EDITOR = "description-icons-editor",
  DESCRIPTION_ICONS_PACKER = "description-icons-packer",
  DESCRIPTION_ICONS_UNPACKER = "description-icons-unpacker",
  DIALOGS_EDITOR = "dialogs-editor",
  EQUIPMENT_ICONS_EDITOR = "equipment-icons-editor",
  EQUIPMENT_ICONS_PACKER = "equipment-icons-packer",
  EQUIPMENT_ICONS_UNPACKER = "equipment-icons-unpacker",
  EXPORTS_EXPLORER = "exports-explorer",
  INFO_PORTIONS_EXPLORER = "info-portions-explorer",
  SPAWN_EDITOR = "spawn-editor",
  SPAWN_PACKER = "spawn-packer",
  SPAWN_UNPACKER = "spawn-unpacker",
  TASKS_EXPLORER = "tasks-explorer",
  TRANSLATIONS_BUILDER = "translations-builder",
  TRANSLATIONS_EDITOR = "translations-editor",
  TRANSLATIONS_VERIFIER = "translations-verifier",
  VISUALS_EXPLORER = "visuals-explorer",
}

/**
 * The family an application belongs to.
 */
export enum EApplicationGroupId {
  ARCHIVES = "archives",
  CONFIGS = "configs",
  DIALOGS = "dialogs",
  EXPORTS = "exports",
  GAMEPLAY = "gameplay",
  ICONS = "icons",
  SPAWNS = "spawns",
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

export interface IApplicationDescriptor {
  id: EApplicationId;
  group: EApplicationGroupId;
  /** The one name this application answers to, everywhere. */
  label: string;
  description: string;
  icon: ReactElement;
  path: string;
  status: EApplicationStatus;
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
