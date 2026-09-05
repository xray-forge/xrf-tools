// Auto-generated rust bindings. Do not edit it manually.

import { invoke as __TAURI_INVOKE, Channel } from "@tauri-apps/api/core";

import { SpawnConversionRequest, SpawnConversionResult } from "@/core/bindings/types/xrf-app";
import {
  AlifeObject,
  ArtefactSpawnPoint,
  GraphCrossTable,
  GraphEdge,
  GraphHeader,
  GraphLevel,
  GraphLevelPoint,
  GraphVertex,
  Patrol,
  SpawnALifeSpawnsChunk,
  SpawnArtefactSpawnsChunk,
  SpawnGraphsChunk,
  SpawnHeaderChunk,
  SpawnPatrolsChunk,
} from "@/core/bindings/types/xrf-db";
import { JobProgress } from "@/core/bindings/types/xrf-job";

/** Commands */
export const spawnCommands = {
  saveUnpackedDirectory: (path: string) => __TAURI_INVOKE<null>("plugin:spawn|save_unpacked_directory", { path }),
  closeFile: () => __TAURI_INVOKE<null>("plugin:spawn|close_file"),
  getFile: () =>
    __TAURI_INVOKE<{
      header: SpawnHeaderChunk;
      alifeSpawn: SpawnALifeSpawnsChunk;
      artefactSpawn: SpawnArtefactSpawnsChunk;
      patrols: SpawnPatrolsChunk;
      graphs: SpawnGraphsChunk;
    } | null>("plugin:spawn|get_file"),
  getAlifeSpawns: () =>
    __TAURI_INVOKE<{
      objects: Array<AlifeObject>;
    } | null>("plugin:spawn|get_alife_spawns"),
  getArtefactSpawns: () =>
    __TAURI_INVOKE<{
      nodes: Array<ArtefactSpawnPoint>;
    } | null>("plugin:spawn|get_artefact_spawns"),
  getGraphs: () =>
    __TAURI_INVOKE<{
      header: GraphHeader;
      levels: Array<GraphLevel>;
      vertices: Array<GraphVertex>;
      edges: Array<GraphEdge>;
      points: Array<GraphLevelPoint>;
      crossTables: Array<GraphCrossTable>;
    } | null>("plugin:spawn|get_graphs"),
  getHeader: () =>
    __TAURI_INVOKE<{
      version: number;
      guid: string;
      graphGuid: string;
      objectsCount: number;
      levelsCount: number;
    } | null>("plugin:spawn|get_header"),
  getPatrols: () =>
    __TAURI_INVOKE<{
      patrols: Array<Patrol>;
    } | null>("plugin:spawn|get_patrols"),
  /** Where the open file came from, so a restored session can name what it is showing. */
  getPath: () => __TAURI_INVOKE<string | null>("plugin:spawn|get_path"),
  hasFile: () => __TAURI_INVOKE<boolean>("plugin:spawn|has_file"),
  openUnpackedDirectory: (path: string) => __TAURI_INVOKE<string>("plugin:spawn|open_unpacked_directory", { path }),
  /**
   * Read a packed spawn file into the application session.
   *
   * Answers with the header rather than the whole file: the UI reads chunks one at a time through the
   * per-chunk commands, so serialising every alife object here only to have it re-requested is waste
   * measured in tens of megabytes on a real all.spawn.
   */
  openFile: (path: string) => __TAURI_INVOKE<SpawnHeaderChunk>("plugin:spawn|open_file", { path }),
  /** Pack a spawn file as an exclusive, tracked background job. */
  packFile: (request: SpawnConversionRequest, jobId: string, progress: Channel<JobProgress>) =>
    __TAURI_INVOKE<SpawnConversionResult>("plugin:spawn|pack_file", { request, jobId, progress }),
  saveFile: (path: string) => __TAURI_INVOKE<null>("plugin:spawn|save_file", { path }),
  /** Unpack a spawn file as an exclusive, tracked background job. */
  unpackFile: (request: SpawnConversionRequest, jobId: string, progress: Channel<JobProgress>) =>
    __TAURI_INVOKE<SpawnConversionResult>("plugin:spawn|unpack_file", { request, jobId, progress }),
};
