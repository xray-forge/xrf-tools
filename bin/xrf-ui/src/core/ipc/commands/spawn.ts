// Auto-generated rust bindings. Do not edit it manually.

import { Channel } from "@tauri-apps/api/core";

import { invoke as __TAURI_INVOKE } from "@/core/ipc/invoke";
import {
  SessionId,
  SpawnConversionRequest,
  SpawnConversionResult,
  SpawnSessionDescriptor,
} from "@/core/ipc/types/xrf-app";
import {
  SpawnALifeSpawnsChunk,
  SpawnArtefactSpawnsChunk,
  SpawnFile,
  SpawnGraphsChunk,
  SpawnHeaderChunk,
  SpawnPatrolsChunk,
} from "@/core/ipc/types/xrf-db";
import { JobProgress } from "@/core/ipc/types/xrf-job";

/** Commands */
export const spawnCommands = {
  /** Write the requested session using the existing spawn format writer. */
  saveUnpackedDirectory: (path: string, sessionId: SessionId) =>
    __TAURI_INVOKE<null>("plugin:spawn|save_unpacked_directory", { path, sessionId }),
  /** Close the committed file and prevent unfinished opens from restoring it. */
  closeFile: (sessionIds: Array<SessionId>) => __TAURI_INVOKE<null>("plugin:spawn|close_file", { sessionIds }),
  /** Read the whole file from the requested opening, refusing a replaced session. */
  getFile: (sessionId: SessionId) => __TAURI_INVOKE<SpawnFile>("plugin:spawn|get_file", { sessionId }),
  /** Read alife_spawn from the requested opening, refusing a replaced session. */
  getAlifeSpawns: (sessionId: SessionId) =>
    __TAURI_INVOKE<SpawnALifeSpawnsChunk>("plugin:spawn|get_alife_spawns", { sessionId }),
  /** Read artefact_spawn from the requested opening, refusing a replaced session. */
  getArtefactSpawns: (sessionId: SessionId) =>
    __TAURI_INVOKE<SpawnArtefactSpawnsChunk>("plugin:spawn|get_artefact_spawns", { sessionId }),
  /** Read graphs from the requested opening, refusing a replaced session. */
  getGraphs: (sessionId: SessionId) => __TAURI_INVOKE<SpawnGraphsChunk>("plugin:spawn|get_graphs", { sessionId }),
  /** Read header from the requested opening, refusing a replaced session. */
  getHeader: (sessionId: SessionId) => __TAURI_INVOKE<SpawnHeaderChunk>("plugin:spawn|get_header", { sessionId }),
  /** Read patrols from the requested opening, refusing a replaced session. */
  getPatrols: (sessionId: SessionId) => __TAURI_INVOKE<SpawnPatrolsChunk>("plugin:spawn|get_patrols", { sessionId }),
  /** Restore a coherent session without cloning any of the large chunks. */
  getSession: () =>
    __TAURI_INVOKE<{
      sessionId: SessionId;
      path: string;
      header: SpawnHeaderChunk;
    } | null>("plugin:spawn|get_session"),
  /** Open a unpacked spawn and return its identity, path, and header together. */
  openUnpackedDirectory: (sessionId: SessionId, path: string) =>
    __TAURI_INVOKE<SpawnSessionDescriptor>("plugin:spawn|open_unpacked_directory", { sessionId, path }),
  /** Open a packed spawn and return its identity, path, and header together. */
  openFile: (sessionId: SessionId, path: string) =>
    __TAURI_INVOKE<SpawnSessionDescriptor>("plugin:spawn|open_file", { sessionId, path }),
  /** Pack a spawn file as an exclusive, tracked background job. */
  packFile: (request: SpawnConversionRequest, jobId: string, progress: Channel<JobProgress>) =>
    __TAURI_INVOKE<SpawnConversionResult>("plugin:spawn|pack_file", { request, jobId, progress }),
  /** Write the requested session using the existing spawn format writer. */
  saveFile: (path: string, sessionId: SessionId) => __TAURI_INVOKE<null>("plugin:spawn|save_file", { path, sessionId }),
  /** Unpack a spawn file as an exclusive, tracked background job. */
  unpackFile: (request: SpawnConversionRequest, jobId: string, progress: Channel<JobProgress>) =>
    __TAURI_INVOKE<SpawnConversionResult>("plugin:spawn|unpack_file", { request, jobId, progress }),
};
