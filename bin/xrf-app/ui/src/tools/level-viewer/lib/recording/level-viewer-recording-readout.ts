import { formatDuration } from "@xrf/format";
import { RecordingOutcome, RecordingProgress } from "@xrf/ipc/types/xrf-app-levels";
import { Nullable } from "@xrf/types";

/** What a recording dropped, running or written: frames, and blocks of sound. */
export type TRecordingDrops = Pick<RecordingOutcome, "droppedFrames" | "droppedSound">;

/** How a recording reads to the user, running in the status bar and written in its toast and notice. */
export class LevelViewerRecordingReadout {
  /**
   * @param progress - Where a recording stands, null before its first word.
   * @returns How long it has run and what it dropped.
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
   * @param outcome - A recording written.
   * @returns Its length, size and what it dropped.
   */
  public static describeOutcome(outcome: RecordingOutcome): string {
    return (
      `${LevelViewerRecordingReadout.describeLength(outcome.duration)} · ${outcome.width} × ${outcome.height} · ` +
      LevelViewerRecordingReadout.describeDrops(outcome)
    );
  }

  /**
   * @param drops - What a recording dropped.
   * @returns The frames and sound dropped, or that nothing was.
   */
  public static describeDrops(drops: TRecordingDrops): string {
    if (!LevelViewerRecordingReadout.hasDrops(drops)) {
      return "nothing dropped";
    }

    return `${drops.droppedFrames} frames and ${drops.droppedSound} sound blocks dropped`;
  }

  /**
   * @param drops - What a recording dropped.
   * @returns Whether it dropped a frame or a block of sound.
   */
  public static hasDrops(drops: TRecordingDrops): boolean {
    return drops.droppedFrames + drops.droppedSound > 0;
  }

  /**
   * @param file - A recording's path.
   * @returns Its file's name.
   */
  public static getFileName(file: string): string {
    return file.split(/[\\/]/).at(-1) ?? file;
  }

  /**
   * @param seconds - A recording's length, null before it is known.
   * @returns It as a duration.
   */
  private static describeLength(seconds: Nullable<number>): string {
    return formatDuration((seconds ?? 0) * 1000);
  }
}
