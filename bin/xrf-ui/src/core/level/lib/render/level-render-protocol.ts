import { Nullable } from "@xrf/types";

import { IBulkRequest } from "@/core/ipc/bulk";
import {
  LevelDetailsDescription,
  LevelSpawnModelDescription,
  LevelSpawnObjectsDescription,
} from "@/core/ipc/types/xrf-app";
import { SectorDescription } from "@/core/ipc/types/xrf-visual";

/** One sector handed to whatever draws it: what the pack says, and the bytes it was packed into. */
export interface ILevelSectorDelivery {
  sector: number;
  description: SectorDescription;
  /** The pack itself, which is the only thing here that is worth transferring rather than copying. */
  buffer: ArrayBuffer;
}

/** What changed about the sectors a level holds. */
export interface ILevelSectorChange {
  delivered: ReadonlyArray<ILevelSectorDelivery>;
  /** Sectors released, or null where every one of them went, which is a level opening or closing. */
  released: Nullable<ReadonlyArray<number>>;
}

/** Told what changed, so whatever draws the level adds and removes exactly that. */
export type TLevelSectorListener = (change: ILevelSectorChange) => void;

/**
 * Sectors as whatever draws them takes them.
 */
export interface ILevelSectorSource {
  /**
   * @param listener - Told what changed, from inside the call that changed it.
   * @returns Stops the telling.
   */
  subscribe(listener: TLevelSectorListener): () => void;
}

/** A level's grass handed to whatever draws it: what the pack says, and the bytes it was packed into. */
export interface ILevelGrassDelivery {
  description: LevelDetailsDescription;
  /** The pack, kept by the loader: whoever draws it takes a copy, so a renderer started later gets it too. */
  buffer: ArrayBuffer;
}

/** One visual a level's spawned objects stand as: what the backend said of it, and its pack. */
export interface ILevelSpawnModel {
  description: LevelSpawnModelDescription;
  /** The pack, kept by the loader: whoever draws it takes a copy, so a renderer started later gets it too. */
  buffer: ArrayBuffer;
}

/**
 * A level's spawned objects handed to whatever draws them: every object from the first delivery on, and the models
 * read so far, which only grow while the objects stay the same.
 */
export interface ILevelSpawnDelivery {
  objects: LevelSpawnObjectsDescription;
  /** By the index of their visual among the objects' visuals. */
  models: ReadonlyMap<number, ILevelSpawnModel>;
  /**
   * Each object's hemisphere cube, six faces, by its index among the level's spawned objects: how the level lights it,
   * delivered with its visual's model. None for an object lit as if under the open sky.
   */
  hemi: ReadonlyMap<number, ReadonlyArray<number>>;
}

/** Told a held value, or null for none. */
export type TLevelHeldListener<T> = (value: Nullable<T>) => void;

/**
 * Something a level reads once, after it opens, as whatever draws it takes it: its grass, its lights, the models its
 * spawned objects stand as.
 */
export interface ILevelHeldSource<T> {
  /** What is held now, or null for nothing yet. */
  readonly held: Nullable<T>;
  /**
   * @param listener - Told what is held now, then whenever it changes.
   * @returns Stops the telling.
   */
  subscribe(listener: TLevelHeldListener<T>): () => void;
}

/** Where the renderer fetches one texture from: the file as stored, and the backend's picture of it. */
export interface ILevelTextureRequests {
  file: IBulkRequest;
  /** Fetched only where the renderer cannot read the file's layout as stored. */
  picture: IBulkRequest;
}

/** One texture handed to whatever uploads it: where the renderer fetches it, not its bytes. */
export interface ILevelTextureDelivery {
  /** The reference as the shader table spells it. */
  reference: string;
  /** Where its file is fetched from, or null for a reference that resolved to nothing. */
  requests: Nullable<ILevelTextureRequests>;
  /** Why there is nothing to fetch, for a reference that could not be asked for at all. */
  reason: Nullable<string>;
}

/** What changed about the textures a level holds. */
export interface ILevelTextureSupplyChange {
  delivered: ReadonlyArray<ILevelTextureDelivery>;
  /**
   * References still worth keeping, everything else being released; null releases nothing, and with nothing delivered
   * says the whole set went.
   */
  retained: Nullable<ReadonlySet<string>>;
}

/** Told what changed, so whatever uploads them uploads exactly that and releases the rest. */
export type TLevelTextureSupplyListener = (change: ILevelTextureSupplyChange) => void;

/** Textures as whatever uploads them takes them: where to fetch each, never a texture. */
export interface ILevelTextureSupply {
  /**
   * @param listener - Told what changed, from inside the call that changed it.
   * @returns Stops the telling.
   */
  subscribe(listener: TLevelTextureSupplyListener): () => void;
}
