// Auto-generated rust bindings. Do not edit it manually.

import { JobOutcome } from "@/core/ipc/types/xrf-job";
import {
  SpawnALifeSpawnsChunk,
  SpawnArtefactSpawnsChunk,
  SpawnGraphsChunk,
  SpawnHeaderChunk,
  SpawnPatrolsChunk,
} from "@/core/ipc/types/xrf-spawn";

/** Packing a directory of spawn chunks into one file. */
export const SPAWN_JOB_PACK = "spawn.pack" as const;

/** Unpacking a spawn file into a directory of chunks. */
export const SPAWN_JOB_UNPACK = "spawn.unpack" as const;

/** One large chunk of an opened spawn, tagged by its `kind` as [`crate::SpawnChunkKind`] names it. */
export type SpawnChunk =
  | ({
      kind: "alifeSpawn";
    } & SpawnALifeSpawnsChunk)
  | ({
      kind: "artefactSpawn";
    } & SpawnArtefactSpawnsChunk)
  | ({
      kind: "patrols";
    } & SpawnPatrolsChunk)
  | ({
      kind: "graphs";
    } & SpawnGraphsChunk);

/** Which of an opened spawn's large chunks to read. */
export enum ESpawnChunkKind {
  ALIFE_SPAWN = "alifeSpawn",
  ARTEFACT_SPAWN = "artefactSpawn",
  PATROLS = "patrols",
  GRAPHS = "graphs",
}

/** Every `ESpawnChunkKind` as the spelling it crosses IPC as, for a value no member has narrowed. */
export type SpawnChunkKind = `${ESpawnChunkKind}`;

/** The conversion performed, retained with the result after a window reload. */
export enum ESpawnConversion {
  PACK = "pack",
  UNPACK = "unpack",
}

/** Every `ESpawnConversion` as the spelling it crosses IPC as, for a value no member has narrowed. */
export type SpawnConversion = `${ESpawnConversion}`;

/** Source and output paths of a standalone spawn conversion. */
export type SpawnConversionRequest = {
  source: string;
  destination: string;
};

/** What reached disk; cancellation is accepted only before writing begins. */
export type SpawnConversionResult = {
  operation: SpawnConversion;
  destination: string;
  outcome: JobOutcome;
};

/**
 * What an opened spawn is, without its large chunks: where it was read from and its header, which an open and a
 * restore answer under the opening's identity.
 */
export type SpawnSessionDescriptor = {
  path: string;
  header: SpawnHeaderChunk;
};
