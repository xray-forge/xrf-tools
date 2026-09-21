import { Nullable } from "@/lib/types/general";

/**
 * What one stage of reading a sector cost, in milliseconds.
 */
export interface ILevelStreamStages {
  /** Packing it on the backend, which is the `open_sector` round trip. */
  pack: number;
  /** Moving its bytes across, which is the `read_sector` round trip. */
  transfer: number;
  /** Binding views over those bytes, which copies nothing and should stay near zero. */
  views: number;
  /** Reading and uploading every texture it names that was not already held. */
  textures: number;
  /** Building its geometry and handing it to the scene. */
  adopt: number;
  /** The whole read, which is the four above plus whatever is between them. */
  total: number;
}

/** What one sector cost, and how much of it there was to pay for. */
export interface ILevelStreamReading extends ILevelStreamStages {
  sector: number;
  vertices: number;
  /** Draws the sector adds, its own sections and its instanced meshes together. */
  draws: number;
}

/** What the reads so far came to, for a viewer to report rather than a log to be grepped. */
export interface ILevelStreamSummary {
  /** Sectors read since the level opened, which the mean is over at most the last {@link STREAM_WINDOW} of. */
  sectors: number;
  last: Nullable<ILevelStreamReading>;
  /** Mean of the window, or null before anything has been read. */
  mean: Nullable<ILevelStreamStages>;
  /** The slowest single read of the window, which is what a stutter is made of. */
  worst: Nullable<ILevelStreamReading>;
}

/** Reads the mean is taken over. Long enough to survive one sector being unusual, short enough to follow the flight. */
export const STREAM_WINDOW: number = 32;

/** Nothing read yet, which is also what a closed level reports. */
export const EMPTY_LEVEL_STREAM_SUMMARY: ILevelStreamSummary = { last: null, mean: null, sectors: 0, worst: null };

/** The stages, in the order they happen, so a reader prints them as a pipeline rather than as a bag. */
export const LEVEL_STREAM_STAGES: ReadonlyArray<keyof ILevelStreamStages> = [
  "pack",
  "transfer",
  "views",
  "textures",
  "adopt",
];

/**
 * Keeps what the recent sector reads cost.
 */
export class LevelStreamProfile {
  private readonly window: Array<ILevelStreamReading> = [];

  private counted: number = 0;

  /**
   * Takes one sector's reading.
   *
   * @param reading - What that sector cost.
   */
  public record(reading: ILevelStreamReading): void {
    this.counted += 1;
    this.window.push(reading);

    if (this.window.length > STREAM_WINDOW) {
      this.window.shift();
    }
  }

  /** Forgets everything, for a level closing or another opening. */
  public clear(): void {
    this.window.length = 0;
    this.counted = 0;
  }

  /**
   * @returns What the window came to, as a value a view can hold.
   */
  public summarise(): ILevelStreamSummary {
    if (!this.window.length) {
      return EMPTY_LEVEL_STREAM_SUMMARY;
    }

    const mean: ILevelStreamStages = { adopt: 0, pack: 0, textures: 0, total: 0, transfer: 0, views: 0 };

    for (const reading of this.window) {
      for (const stage of LEVEL_STREAM_STAGES) {
        mean[stage] += reading[stage];
      }

      mean.total += reading.total;
    }

    for (const stage of LEVEL_STREAM_STAGES) {
      mean[stage] /= this.window.length;
    }

    mean.total /= this.window.length;

    return {
      last: this.window[this.window.length - 1],
      mean,
      sectors: this.counted,
      worst: this.window.reduce((worst, it) => (it.total > worst.total ? it : worst)),
    };
  }
}
