import { SectorSkip } from "@/core/ipc/types/xrf-visual";

/** One drawable a sector's pack could not read, and the sector it belongs to. */
export interface ILevelSectorSkip {
  sector: number;
  skip: SectorSkip;
}

/** What a level is holding, as data, for everything that reports on it and draws none of it. */
export interface ILevelSectorReport {
  /** Sectors held, by index, nearest first as the plan ordered them. */
  held: ReadonlyArray<number>;
  /** Bytes of packed geometry held, which is what the memory budget is spending. */
  bytes: number;
  /** Everything the packs could not read, which is geometry simply absent from the picture. */
  skipped: ReadonlyArray<ILevelSectorSkip>;
}

/** Nothing held, which is also what a closed level reports. */
export const EMPTY_LEVEL_SECTOR_REPORT: ILevelSectorReport = { bytes: 0, held: [], skipped: [] };
