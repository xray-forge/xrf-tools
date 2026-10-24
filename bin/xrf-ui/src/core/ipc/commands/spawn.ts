// Auto-generated rust bindings. Do not edit it manually.

import { Channel } from "@tauri-apps/api/core";

import { invoke as __TAURI_INVOKE } from "@/core/ipc/invoke";
import { IIpcCommandSignature } from "@/core/ipc/signature";
import { SessionId, SessionRestore, SessionSnapshot } from "@/core/ipc/types/xrf-app-core";
import {
  ESpawnChunkKind,
  SpawnChunk,
  SpawnConversionRequest,
  SpawnConversionResult,
  SpawnSessionDescriptor,
} from "@/core/ipc/types/xrf-app-spawn";
import { JobProgress } from "@/core/ipc/types/xrf-job";

/** The wire name of every command of the `spawn` plugin. */
export enum ESpawnCommand {
  CLOSE_FILE = "plugin:spawn|close_file",
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
  /** Close the spawns `session_ids` name, committed or opening; a closed one answers no read. */
  closeFile: (sessionIds: Array<SessionId>) => __TAURI_INVOKE<void>(ESpawnCommand.CLOSE_FILE, { sessionIds }),
  /** Open the packed spawn at `path`, read on the pool, answering what describes it without its large chunks. */
  openFile: (sessionId: SessionId, path: string) =>
    __TAURI_INVOKE<SessionSnapshot<SpawnSessionDescriptor>>(ESpawnCommand.OPEN_FILE, { sessionId, path }),
  /** Pack a directory of spawn chunks into one file, as a tracked job sharing one lane with unpacking. */
  packFile: (request: SpawnConversionRequest, jobId: string, progress: Channel<JobProgress>) =>
    __TAURI_INVOKE<SpawnConversionResult>(ESpawnCommand.PACK_FILE, { request, jobId, progress }),
  /** One large chunk of the spawn `session_id` holds, copied on the pool. */
  readChunk: (sessionId: SessionId, chunk: ESpawnChunkKind) =>
    __TAURI_INVOKE<SessionSnapshot<SpawnChunk>>(ESpawnCommand.READ_CHUNK, { sessionId, chunk }),
  /** The spawn `tool` holds in the asking window, the newest it opened, described without its large chunks. */
  restoreFile: (tool: string) =>
    __TAURI_INVOKE<SessionRestore<SpawnSessionDescriptor>>(ESpawnCommand.RESTORE_FILE, { tool }),
  /** Write the spawn `session_id` holds to `path` as a packed file, on the pool. */
  saveFile: (path: string, sessionId: SessionId) => __TAURI_INVOKE<null>(ESpawnCommand.SAVE_FILE, { path, sessionId }),
  /** Write the spawn `session_id` holds into the directory `path` as unpacked chunks, on the pool. */
  saveUnpackedDirectory: (path: string, sessionId: SessionId) =>
    __TAURI_INVOKE<null>(ESpawnCommand.SAVE_UNPACKED_DIRECTORY, { path, sessionId }),
  /** Unpack a spawn file into a directory of chunks, as a tracked job sharing one lane with packing. */
  unpackFile: (request: SpawnConversionRequest, jobId: string, progress: Channel<JobProgress>) =>
    __TAURI_INVOKE<SpawnConversionResult>(ESpawnCommand.UNPACK_FILE, { request, jobId, progress }),
};

/** What each command of the `spawn` plugin takes and resolves to, keyed by its wire name. */
export interface ISpawnCommandSignatures {
  [ESpawnCommand.CLOSE_FILE]: IIpcCommandSignature<typeof spawnCommands.closeFile, ["sessionIds"]>;
  [ESpawnCommand.OPEN_FILE]: IIpcCommandSignature<typeof spawnCommands.openFile, ["sessionId", "path"]>;
  [ESpawnCommand.PACK_FILE]: IIpcCommandSignature<typeof spawnCommands.packFile, ["request", "jobId", "progress"]>;
  [ESpawnCommand.READ_CHUNK]: IIpcCommandSignature<typeof spawnCommands.readChunk, ["sessionId", "chunk"]>;
  [ESpawnCommand.RESTORE_FILE]: IIpcCommandSignature<typeof spawnCommands.restoreFile, ["tool"]>;
  [ESpawnCommand.SAVE_FILE]: IIpcCommandSignature<typeof spawnCommands.saveFile, ["path", "sessionId"]>;
  [ESpawnCommand.SAVE_UNPACKED_DIRECTORY]: IIpcCommandSignature<
    typeof spawnCommands.saveUnpackedDirectory,
    ["path", "sessionId"]
  >;
  [ESpawnCommand.UNPACK_FILE]: IIpcCommandSignature<typeof spawnCommands.unpackFile, ["request", "jobId", "progress"]>;
}
