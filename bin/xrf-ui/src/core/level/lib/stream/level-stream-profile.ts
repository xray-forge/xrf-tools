import { toMean } from "@xrf/math";
import { Nullable } from "@xrf/types";

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
  /** Reading every texture it names that was not already held. */
  textures: number;
  /** Handing it on to whatever draws it, which is what builds its geometry. */
  deliver: number;
  /** The whole read, which is the four above plus whatever is between them. */
  total: number;
}

/** What one sector cost, and how much of it there was to pay for. */
export interface ILevelStreamReading extends ILevelStreamStages {
  sector: number;
  /** Texture files this sector was the first to ask for, which is what its `textures` stage bought. */
  files: number;
  vertices: number;
  /** Draws the sector adds, its own sections and its instanced meshes together. */
  draws: number;
}

/**
 * What a camera report costs.
 *
 * Kept apart from the readings because it is paid on a different schedule: a reading happens once per sector, a
 * report happens every time the camera has moved far enough, which under boost is every frame.
 */
export interface ILevelStreamPlanning {
  /** Camera reports since the level opened. */
  reports: number;
  /** What the recent ones cost, in milliseconds. */
  mean: number;
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
  /** What answering the camera costs, which no reading covers. */
  planning: ILevelStreamPlanning;
}

/** Reads the mean is taken over. Long enough to survive one sector being unusual, short enough to follow the flight. */
export const STREAM_WINDOW: number = 32;

/** Nothing read yet, which is also what a closed level reports. */
export const EMPTY_LEVEL_STREAM_SUMMARY: ILevelStreamSummary = {
  last: null,
  mean: null,
  planning: { mean: 0, reports: 0 },
  sectors: 0,
  worst: null,
};

/** The stages, in the order they happen, so a reader prints them as a pipeline rather than as a bag. */
export const LEVEL_STREAM_STAGES: ReadonlyArray<keyof ILevelStreamStages> = [
  "pack",
  "transfer",
  "views",
  "textures",
  "deliver",
];

/**
 * Keeps what the recent sector reads cost.
 */
export class LevelStreamProfile {
  private readonly window: Array<ILevelStreamReading> = [];
  private readonly reports: Array<number> = [];

  private counted: number = 0;
  private reported: number = 0;

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

  /**
   * Takes what one camera report cost, which is planning and whatever answering it started.
   *
   * @param elapsed - Milliseconds the report took on the thread that draws.
   */
  public recordReport(elapsed: number): void {
    this.reported += 1;
    this.reports.push(elapsed);

    if (this.reports.length > STREAM_WINDOW) {
      this.reports.shift();
    }
  }

  /** Forgets everything, for a level closing or another opening. */
  public clear(): void {
    this.window.length = 0;
    this.reports.length = 0;
    this.counted = 0;
    this.reported = 0;
  }

  /**
   * @returns What the window came to, as a value a view can hold.
   */
  public summarise(): ILevelStreamSummary {
    const planning: ILevelStreamPlanning = {
      mean: toMean(this.reports),
      reports: this.reported,
    };

    if (!this.window.length) {
      return { ...EMPTY_LEVEL_STREAM_SUMMARY, planning };
    }

    const mean: ILevelStreamStages = {
      deliver: 0,
      pack: 0,
      textures: 0,
      total: 0,
      transfer: 0,
      views: 0,
    };

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
      planning,
      sectors: this.counted,
      worst: this.window.reduce((worst, it) => (it.total > worst.total ? it : worst)),
    };
  }
}
