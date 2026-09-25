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

/** Told the level's grass, or null for none. */
export type TLevelGrassListener = (grass: Nullable<ILevelGrassDelivery>) => void;

/**
 * The level's grass as whatever draws it takes it.
 */
export interface ILevelGrassSource {
  /**
   * @param listener - Told the grass held now, then whenever it changes.
   * @returns Stops the telling.
   */
  subscribe(listener: TLevelGrassListener): () => void;
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
}

/** What changed about the textures a level holds. */
export interface ILevelTextureSupplyChange {
  delivered: ReadonlyArray<ILevelTextureDelivery>;
  /** References still worth keeping, everything else being released, or null where the whole set went. */
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
