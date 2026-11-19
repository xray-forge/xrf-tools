// Auto-generated rust bindings. Do not edit it manually.

import { Channel } from "@tauri-apps/api/core";

import { invoke as __TAURI_INVOKE } from "#/invoke";
import { IIpcCommandSignature } from "#/signature";
import { SessionId, SessionRestore, SessionSnapshot } from "#/types/xrf-app-core";
import {
  ESpawnChunkKind,
  SpawnBuildResult,
  SpawnChunk,
  SpawnConversionResult,
  SpawnConvertRequest,
  SpawnSessionDescriptor,
} from "#/types/xrf-app-spawn";
import { JobProgress } from "#/types/xrf-job";
import { SpawnListedLevel, SpawnRootRequest } from "#/types/xrf-spawn-builder";

/** The wire name of every command of the `spawn` plugin. */
export enum ESpawnCommand {
  BUILD_SPAWN = "plugin:spawn|build_spawn",
  CLOSE_FILE = "plugin:spawn|close_file",
  LIST_LEVELS = "plugin:spawn|list_levels",
  OPEN_FILE = "plugin:spawn|open_file",
  PACK_FILE = "plugin:spawn|pack_file",
  READ_CHUNK = "plugin:spawn|read_chunk",
  RESTORE_FILE = "plugin:spawn|restore_file",
  SAVE_FILE = "plugin:spawn|save_file",
  SAVE_UNPACKED_DIRECTORY = "plugin:spawn|save_unpacked_directory",
  UNPACK_FILE = "plugin:spawn|unpack_file",
}

/** Commands */
export const spawnCommands = {
  /** Build and write a root spawn from a game's levels, as `spawn build` does, as a job cancellable until it writes. */
  buildSpawn: (request: SpawnRootRequest, jobId: string, progress: Channel<JobProgress>) =>
    __TAURI_INVOKE<SpawnBuildResult>(ESpawnCommand.BUILD_SPAWN, { request, jobId, progress }),
  /** Close the spawns `session_ids` name, committed or opening; a closed one answers no read. */
  closeFile: (sessionIds: Array<SessionId>) => __TAURI_INVOKE<void>(ESpawnCommand.CLOSE_FILE, { sessionIds }),
  /**
   * The levels a spawn can be built from: every level `gamedata`'s `configs/game_levels.ltx` lists, in its order, each
   * marked by whether its directory under `levels_root` holds `level.ai`, `level.spawn` and `level.game`.
   */
  listLevels: (gamedata: string, levelsRoot: string) =>
    __TAURI_INVOKE<Array<SpawnListedLevel>>(ESpawnCommand.LIST_LEVELS, { gamedata, levelsRoot }),
  /** Open the packed spawn at `path`, read on the pool, answering what describes it without its large chunks. */
  openFile: (sessionId: SessionId, path: string) =>
    __TAURI_INVOKE<SessionSnapshot<SpawnSessionDescriptor>>(ESpawnCommand.OPEN_FILE, { sessionId, path }),
  /** Pack a directory of spawn chunks into one file, as a tracked job sharing one lane with unpacking. */
  packFile: (request: SpawnConvertRequest, jobId: string, progress: Channel<JobProgress>) =>
    __TAURI_INVOKE<SpawnConversionResult>(ESpawnCommand.PACK_FILE, { request, jobId, progress }),
  /** One large chunk of the spawn `session_id` holds, copied on the pool. */
  readChunk: (sessionId: SessionId, chunk: ESpawnChunkKind) =>
    __TAURI_INVOKE<SessionSnapshot<SpawnChunk>>(ESpawnCommand.READ_CHUNK, { sessionId, chunk }),
  /** The spawn `tool` holds in the asking window, the newest it opened, described without its large chunks. */
  restoreFile: (tool: string) =>
    __TAURI_INVOKE<SessionRestore<SpawnSessionDescriptor>>(ESpawnCommand.RESTORE_FILE, { tool }),
  /**
   * Write the spawn `session_id` holds to `path` as a packed file; refused where `path` exists, in any letter case,
   * unless `is_forced`.
   */
  saveFile: (path: string, sessionId: SessionId, isForced: boolean) =>
    __TAURI_INVOKE<null>(ESpawnCommand.SAVE_FILE, { path, sessionId, isForced }),
  /**
   * Write the spawn `session_id` holds into the directory `path` as unpacked chunks; refused where `path` exists, in
   * any letter case, unless `is_forced`.
   */
  saveUnpackedDirectory: (path: string, sessionId: SessionId, isForced: boolean) =>
    __TAURI_INVOKE<null>(ESpawnCommand.SAVE_UNPACKED_DIRECTORY, { path, sessionId, isForced }),
  /** Unpack a spawn file into a directory of chunks, as a tracked job sharing one lane with packing. */
  unpackFile: (request: SpawnConvertRequest, jobId: string, progress: Channel<JobProgress>) =>
    __TAURI_INVOKE<SpawnConversionResult>(ESpawnCommand.UNPACK_FILE, { request, jobId, progress }),
};

/** What each command of the `spawn` plugin takes and resolves to, keyed by its wire name. */
export interface ISpawnCommandSignatures {
  [ESpawnCommand.BUILD_SPAWN]: IIpcCommandSignature<typeof spawnCommands.buildSpawn, ["request", "jobId", "progress"]>;
  [ESpawnCommand.CLOSE_FILE]: IIpcCommandSignature<typeof spawnCommands.closeFile, ["sessionIds"]>;
  [ESpawnCommand.LIST_LEVELS]: IIpcCommandSignature<typeof spawnCommands.listLevels, ["gamedata", "levelsRoot"]>;
  [ESpawnCommand.OPEN_FILE]: IIpcCommandSignature<typeof spawnCommands.openFile, ["sessionId", "path"]>;
  [ESpawnCommand.PACK_FILE]: IIpcCommandSignature<typeof spawnCommands.packFile, ["request", "jobId", "progress"]>;
  [ESpawnCommand.READ_CHUNK]: IIpcCommandSignature<typeof spawnCommands.readChunk, ["sessionId", "chunk"]>;
  [ESpawnCommand.RESTORE_FILE]: IIpcCommandSignature<typeof spawnCommands.restoreFile, ["tool"]>;
  [ESpawnCommand.SAVE_FILE]: IIpcCommandSignature<typeof spawnCommands.saveFile, ["path", "sessionId", "isForced"]>;
  [ESpawnCommand.SAVE_UNPACKED_DIRECTORY]: IIpcCommandSignature<
    typeof spawnCommands.saveUnpackedDirectory,
    ["path", "sessionId", "isForced"]
  >;
  [ESpawnCommand.UNPACK_FILE]: IIpcCommandSignature<typeof spawnCommands.unpackFile, ["request", "jobId", "progress"]>;
}
