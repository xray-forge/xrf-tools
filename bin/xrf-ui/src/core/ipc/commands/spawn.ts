// Auto-generated rust bindings. Do not edit it manually.

import { Channel } from "@tauri-apps/api/core";

import { invoke as __TAURI_INVOKE } from "@/core/ipc/invoke";
import {
  ESpawnChunkKind,
  SessionId,
  SessionRestore,
  SessionSnapshot,
  SpawnChunk,
  SpawnConversionRequest,
  SpawnConversionResult,
  SpawnSessionDescriptor,
} from "@/core/ipc/types/xrf-app";
import { JobProgress } from "@/core/ipc/types/xrf-job";

/** Commands */
export const spawnCommands = {
  /** Close the spawns `session_ids` name, committed or opening; a closed one answers no read. */
  closeFile: (sessionIds: Array<SessionId>) => __TAURI_INVOKE<void>("plugin:spawn|close_file", { sessionIds }),
  /** Open the packed spawn at `path`, read on the pool, answering what describes it without its large chunks. */
  openFile: (sessionId: SessionId, path: string) =>
    __TAURI_INVOKE<SessionSnapshot<SpawnSessionDescriptor>>("plugin:spawn|open_file", { sessionId, path }),
  /** Pack a directory of spawn chunks into one file, as a tracked job sharing one lane with unpacking. */
  packFile: (request: SpawnConversionRequest, jobId: string, progress: Channel<JobProgress>) =>
    __TAURI_INVOKE<SpawnConversionResult>("plugin:spawn|pack_file", { request, jobId, progress }),
  /** One large chunk of the spawn `session_id` holds, copied on the pool. */
  readChunk: (sessionId: SessionId, chunk: ESpawnChunkKind) =>
    __TAURI_INVOKE<SessionSnapshot<SpawnChunk>>("plugin:spawn|read_chunk", { sessionId, chunk }),
  /** The spawn `tool` holds in the asking window, the newest it opened, described without its large chunks. */
  restoreFile: (tool: string) =>
    __TAURI_INVOKE<SessionRestore<SpawnSessionDescriptor>>("plugin:spawn|restore_file", { tool }),
  /** Write the spawn `session_id` holds to `path` as a packed file, on the pool. */
  saveFile: (path: string, sessionId: SessionId) => __TAURI_INVOKE<null>("plugin:spawn|save_file", { path, sessionId }),
  /** Write the spawn `session_id` holds into the directory `path` as unpacked chunks, on the pool. */
  saveUnpackedDirectory: (path: string, sessionId: SessionId) =>
    __TAURI_INVOKE<null>("plugin:spawn|save_unpacked_directory", { path, sessionId }),
  /** Unpack a spawn file into a directory of chunks, as a tracked job sharing one lane with packing. */
  unpackFile: (request: SpawnConversionRequest, jobId: string, progress: Channel<JobProgress>) =>
    __TAURI_INVOKE<SpawnConversionResult>("plugin:spawn|unpack_file", { request, jobId, progress }),
};
