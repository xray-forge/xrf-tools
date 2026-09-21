import { SectorDescription } from "@/core/ipc/types/xrf-visual";
import { Nullable } from "@/lib/types/general";

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
