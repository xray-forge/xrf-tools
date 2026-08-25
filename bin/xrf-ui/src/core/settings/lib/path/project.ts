import * as path from "@tauri-apps/api/path";
import { exists } from "@tauri-apps/plugin-fs";

import { Nullable } from "@/lib/types/general";

export function getProjectConfigsPath(projectPath: string): Promise<string> {
  return path.resolve(projectPath, "src", "engine", "configs");
}

export function getProjectBuiltAllSpawnPath(projectPath: string): Promise<string> {
  return path.resolve(projectPath, "target", "gamedata", "spawns", "all.spawn");
}

export function getProjectAllSpawnUnpackPath(projectPath: string): Promise<string> {
  return path.resolve(projectPath, "target", "spawns", "unpacked");
}

export function getProjectAllSpawnRepackPath(projectPath: string): Promise<string> {
  return path.resolve(projectPath, "target", "spawns", "repacked", "repacked.spawn");
}

export function getProjectLinkedGamePath(projectPath: string): Promise<string> {
  return path.resolve(projectPath, "target", "game_link");
}

export function getProjectArchivesUnpackPath(projectPath: string): Promise<string> {
  return path.resolve(projectPath, "target", "unpacked_archives");
}

export function getProjectGamedataPath(projectPath: string): Promise<string> {
  return path.resolve(projectPath, "target", "gamedata");
}

export function getProjectArchivesPackPath(projectPath: string): Promise<string> {
  return path.resolve(projectPath, "target", "db");
}

export function getProjectEquipmentDDSPath(projectPath: string): Promise<string> {
  return path.resolve(projectPath, "src", "resources", "textures", "ui", "ui_icon_equipment.dds");
}

export function getProjectEquipmentSourcePath(projectPath: string): Promise<string> {
  return path.resolve(projectPath, "src", "resources", "textures", "ui", "ui_icon_equipment");
}

export function getProjectTranslationsPath(projectPath: string): Promise<string> {
  return path.resolve(projectPath, "src", "engine", "translations");
}

/**
 * The engine source root, which holds `configs` and `translations` as siblings.
 *
 * What a tool mounts, rather than what it reads: a logical prefix narrows to the data from here, so
 * the same root serves dialogs and their text.
 */
export function getProjectEnginePath(projectPath: string): Promise<string> {
  return path.resolve(projectPath, "src", "engine");
}

export async function getProjectSystemLtxPath(projectPath: string): Promise<string> {
  return path.resolve(await getProjectConfigsPath(projectPath), "system.ltx");
}

export async function getExistingProjectBuiltAllSpawnPath(projectPath: string): Promise<Nullable<string>> {
  return getPathIfExists(getProjectBuiltAllSpawnPath(projectPath));
}

export async function getExistingProjectUnpackedAllSpawnPath(projectPath: string): Promise<Nullable<string>> {
  return getPathIfExists(getProjectAllSpawnUnpackPath(projectPath));
}

export async function getExistingProjectLinkedGamePath(projectPath: string): Promise<Nullable<string>> {
  return getPathIfExists(getProjectLinkedGamePath(projectPath));
}

export async function getPathIfExists(path: string | Promise<string>): Promise<Nullable<string>> {
  const resolved: string = await path;

  return (await exists(resolved)) ? resolved : null;
}
