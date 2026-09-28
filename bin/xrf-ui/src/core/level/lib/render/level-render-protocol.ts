import { Nullable } from "@xrf/types";

import { LevelDetailsDescription } from "@/core/ipc/types/xrf-app";
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

/** Told a held value, or null for none. */
export type TLevelHeldListener<T> = (value: Nullable<T>) => void;

/**
 * Something a level reads once, after it opens, as whatever draws it takes it: its grass, its lights, the models its
 * spawned objects stand as.
 */
export interface ILevelHeldSource<T> {
  /**
   * @param listener - Told what is held now, then whenever it changes.
   * @returns Stops the telling.
   */
  subscribe(listener: TLevelHeldListener<T>): () => void;
}

/** How large a texture file is, as its reader found it. */
export interface ILevelTextureSize {
  width: number;
  height: number;
  /** Mip levels it carries. */
  levels: number;
}

/** One texture's file, handed to whatever uploads it. */
export interface ILevelTextureDelivery {
  /** The reference as the shader table spells it. */
  reference: string;
  /** The file itself, which is the only thing here worth transferring rather than copying. */
  bytes: ArrayBuffer;
  /**
   * Whether the bytes are a picture rather than the file the level names.
   *
   * Decided by whoever read it, because deciding it is reading and not drawing: a layout the dds reader does not
   * model is fetched already expanded, so the side that uploads never has to ask for anything.
   */
  isDecoded: boolean;
  /** Why there is no file, for a reference that could not be read at all. */
  reason: Nullable<string>;
  /** Its size as read, for a file the dds reader models; null for a decoded picture or none. */
  size: Nullable<ILevelTextureSize>;
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

/** Textures as whatever uploads them takes them: a file and what to sample it as, never a texture. */
export interface ILevelTextureSupply {
  /**
   * @param listener - Told what changed, from inside the call that changed it.
   * @returns Stops the telling.
   */
  subscribe(listener: TLevelTextureSupplyListener): () => void;
}
