import { ComponentType, ReactElement } from "react";

import { IKeybindCommand } from "@/core/commands/lib/command-descriptor";
import { IMenu } from "@/core/shell/menu/lib/menu-descriptor";
import { IMenuExtension } from "@/core/shell/menu/lib/menu-extension";
import { ContainerDefinition } from "@/lib/container/container-definition";

/** Canonical tool identity shared by its directory, route, descriptor, and component stem. */
export enum EToolId {
  ARCHIVES_EXPLORER = "archives-explorer",
  ARCHIVES_PACKER = "archives-packer",
  ARCHIVES_PATCHER = "archives-patcher",
  ARCHIVES_UNPACKER = "archives-unpacker",
  CHARACTERS_EXPLORER = "characters-explorer",
  CONFIGS_EXPLORER = "configs-explorer",
  CONFIGS_FORMATTER = "configs-formatter",
  CONFIGS_VERIFIER = "configs-verifier",
  DIALOGS_EDITOR = "dialogs-editor",
  EXPORTS_EXPLORER = "exports-explorer",
  GAMEDATA_COMPARER = "gamedata-comparer",
  GAMEDATA_VERIFIER = "gamedata-verifier",
  GAME_MATERIALS_EDITOR = "game-materials-editor",
  INFO_PORTIONS_EXPLORER = "info-portions-explorer",
  LEVEL_AI_COMPILER = "level-ai-compiler",
  LEVEL_COMPILER = "level-compiler",
  LEVEL_DECOMPILER = "level-decompiler",
  LEVEL_DETAILS_COMPILER = "level-details-compiler",
  LEVEL_EDITOR = "level-editor",
  LEVEL_VIEWER = "level-viewer",
  LIGHT_ANIMATIONS_EDITOR = "light-animations-editor",
  LUA_EXPORTS_EXPLORER = "lua-exports-explorer",
  LUA_FORMATTER = "lua-formatter",
  LUA_VERIFIER = "lua-verifier",
  MINIMAP_EDITOR = "minimap-editor",
  OBJECT_LIBRARY_EDITOR = "object-library-editor",
  PARTICLES_EDITOR = "particles-editor",
  PARTICLES_EXPLORER = "particles-explorer",
  POSTPROCESS_EDITOR = "postprocess-editor",
  SHADERS_EDITOR = "shaders-editor",
  SHADERS_EXPLORER = "shaders-explorer",
  SOUNDS_EDITOR = "sounds-editor",
  SOUNDS_EXPLORER = "sounds-explorer",
  SOUND_ENVIRONMENTS_EDITOR = "sound-environments-editor",
  SPAWN_COMPILER = "spawn-compiler",
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
  TEXTURES_EDITOR = "textures-editor",
  TEXTURES_EXPLORER = "textures-explorer",
  TRANSLATIONS_BUILDER = "translations-builder",
  TRANSLATIONS_EDITOR = "translations-editor",
  TRANSLATIONS_FORMATTER = "translations-formatter",
  TRANSLATIONS_PARSER = "translations-parser",
  TRANSLATIONS_VERIFIER = "translations-verifier",
  VISUALS_CONVERTER = "visuals-converter",
  VISUALS_EDITOR = "visuals-editor",
  VISUALS_EXPLORER = "visuals-explorer",
  VISUALS_SEQUENCER = "visuals-sequencer",
  WEATHER_EDITOR = "weather-editor",
}

/**
 * The family a tool belongs to.
 */
export enum EToolGroupId {
  ARCHIVES = "archives",
  CONFIGS = "configs",
  DIALOGS = "dialogs",
  ENVIRONMENT = "environment",
  GAMEDATA = "gamedata",
  GAMEPLAY = "gameplay",
  LEVEL = "level",
  MATERIALS = "materials",
  PARTICLES = "particles",
  SCRIPTS = "scripts",
  SHADERS = "shaders",
  SOUNDS = "sounds",
  SPAWNS = "spawns",
  SPRITES = "sprites",
  TEXTURES = "textures",
  TRANSLATIONS = "translations",
  VISUALS = "visuals",
}

/**
 * Whether a tool does anything yet.
 */
export enum EToolStatus {
  PLANNED = "planned",
  READY = "ready",
}

/**
 * What a tool tells its user about itself, beyond the launcher one-liner.
 *
 * The shape is the rubric: nuances and limitations over a walkthrough of controls the screen already
 * shows. Backticked spans render as code; that is the only markup any string carries.
 */
export interface IToolHelp {
  /** What the tool is for and when to reach for it, in a few sentences. */
  summary: string;
  /** The typical run, as ordered steps. */
  workflow?: ReadonlyArray<string>;
  /** Behaviors worth knowing that the screen does not make obvious. */
  nuances?: ReadonlyArray<string>;
  /** What the tool deliberately does not do, and its known constraints. */
  limitations?: ReadonlyArray<string>;
  /** Tools belonging to the same workflow. */
  relatedTools?: ReadonlyArray<EToolId>;
}

/** The component and service bindings that make up a tool. */
export interface IToolRuntime {
  Component: ComponentType;
  /** Omit the container to use root services only. */
  container?: ContainerDefinition;
}

/** Identity and presentation available before a tool's runtime loads. */
export interface IToolMetadata {
  id: EToolId;
  group: EToolGroupId;
  /** The one name this tool answers to, everywhere. */
  label: string;
  description: string;
  icon: ReactElement;
  path: string;
  status: EToolStatus;
  /** In-tool help. Required for `READY` tools once the roster is covered. */
  help?: IToolHelp;
  /** Every command reachable inside this tool, root ones excluded. */
  keybindCommands?: ReadonlyArray<IKeybindCommand>;
  /** The menus this tool adds between File and Help, in bar order. */
  menus?: ReadonlyArray<IMenu>;
  /** Sections this tool adds to File or Help while it is open. */
  menuExtensions?: ReadonlyArray<IMenuExtension>;
}

export interface IToolDescriptor extends IToolMetadata, IToolRuntime {
  /** Returns the cached runtime promise, including any load failure. */
  load?: () => Promise<IToolRuntime>;
  /** Warms the runtime and resolves silently on failure; opening still reports the error. */
  preload?: () => Promise<void>;
}

export interface IToolGroupAccent {
  light: string;
  dark: string;
}

export interface IToolGroup {
  id: EToolGroupId;
  label: string;
  icon: ReactElement;
  accent: IToolGroupAccent;
}

/** Sources that raise notifications without owning a tool of their own. */
export const APPLICATION_SOURCE: string = "application";
