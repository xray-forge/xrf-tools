import { formatDuration } from "@xrf/format";
import { RecordingOutcome, RecordingProgress } from "@xrf/ipc/types/xrf-app-levels";
import { Nullable } from "@xrf/types";

/** What a recording dropped, running or written: frames, and blocks of sound. */
export type TRecordingDrops = Pick<RecordingOutcome, "droppedFrames" | "droppedSound">;

/** How a recording reads to the user, running in the status bar and written in its toast and notice. */
export class LevelViewerRecordingReadout {
  /**
   * How long a running recording has run and what it dropped.
   *
   * @param progress - Where it stands, null before its first report.
   */
  public static describeProgress(progress: Nullable<RecordingProgress>): string {
    if (!progress) {
      return "Recording…";
    }

    return (
      `Recording ${LevelViewerRecordingReadout.describeLength(progress.elapsed)}, ` +
      LevelViewerRecordingReadout.describeDrops(progress)
    );
  }

  /**
   * A written recording's length, size and what it dropped.
   */
  public static describeOutcome(outcome: RecordingOutcome): string {
    return (
      `${LevelViewerRecordingReadout.describeLength(outcome.duration)} · ${outcome.width} × ${outcome.height} · ` +
      LevelViewerRecordingReadout.describeDrops(outcome)
    );
  }

  /**
   * The frames and sound blocks a recording dropped, or that nothing was.
   */
  public static describeDrops(drops: TRecordingDrops): string {
    if (!LevelViewerRecordingReadout.hasDrops(drops)) {
      return "nothing dropped";
    }

    return `${drops.droppedFrames} frames and ${drops.droppedSound} sound blocks dropped`;
  }

  public static hasDrops(drops: TRecordingDrops): boolean {
    return drops.droppedFrames + drops.droppedSound > 0;
  }

  /**
   * The file name of a recording's path.
   */
  public static getFileName(file: string): string {
    return file.split(/[\\/]/).at(-1) ?? file;
  }

  /**
   * A recording's length as a duration, zero while unknown.
   */
  private static describeLength(seconds: Nullable<number>): string {
    return formatDuration((seconds ?? 0) * 1000);
  }
}
