import { Channel } from "@tauri-apps/api/core";
import { inject, Injectable, OnActivation, OnDeactivation } from "@wirestate/core";
import { BoundAction, Computed, RefObservable } from "@wirestate/mobx";
import { ExclusiveFlow, TFlow } from "@xrf/flow";
import { formatDuration } from "@xrf/format";
import { levelsCommands } from "@xrf/ipc/commands/levels";
import { RecordingAvailability, RecordingOutcome, RecordingProgress } from "@xrf/ipc/types/xrf-app-levels";
import { Logger } from "@xrf/logging";
import { Nullable } from "@xrf/types";

import { LevelRenderService } from "@/core/level/services/level-render";
import { ENotificationSeverity } from "@/core/notifications/lib";
import { CommandReporter, TCommandCompletion } from "@/core/operations/lib/command-reporter";
import { toLevelViewerFileStem } from "@/tools/level-viewer/lib/files/level-viewer-file-stem";
import { LevelViewerSnapshotService } from "@/tools/level-viewer/services/level-viewer-snapshot";

/**
 * The level viewer's recording: the viewport with what its camera hears written into one video in the recordings
 * folder until it is stopped, one at a time, its camera heard whatever window takes the focus meanwhile. Off where this
 * machine cannot record, saying why.
 */
@Injectable()
export class LevelViewerRecordingService {
  public readonly log: Logger = new Logger(__MODULE_NAME__);

  /** Whether this machine can record, once asked; null until then. */
  @RefObservable()
  public availability: Nullable<RecordingAvailability> = null;

  /** The file being recorded into, while a recording runs. */
  @RefObservable()
  public file: Nullable<string> = null;

  /** Where the recording stands, as last told; null before its first word. */
  @RefObservable()
  public progress: Nullable<RecordingProgress> = null;

  /** The last recording written. */
  @RefObservable()
  public outcome: Nullable<RecordingOutcome> = null;

  private readonly reporter: CommandReporter = new CommandReporter(this.log);

  public constructor(
    private readonly renderService: LevelRenderService = inject(LevelRenderService),
    private readonly snapshotService: LevelViewerSnapshotService = inject(LevelViewerSnapshotService)
  ) {}

  @OnActivation()
  public onActivation(): void {
    void this.describe();
  }

  @OnDeactivation()
  public onDeactivation(): void {
    if (this.isRecording) {
      void levelsCommands.stopRecording().catch(() => null);
    }
  }

  /** Whether a recording runs. */
  @Computed()
  public get isRecording(): boolean {
    return this.file !== null;
  }

  /** Whether a recording can start: this machine records, a level is open, and none runs. */
  @Computed()
  public get canStart(): boolean {
    return this.availability?.isAvailable === true && this.snapshotService.level !== null && !this.isRecording;
  }

  /** Starts a recording where none runs, else stops the one running. */
  @ExclusiveFlow()
  public *toggle(): TFlow {
    if (this.isRecording) {
      yield* this.finish();
    } else {
      yield* this.record();
    }
  }

  /** Starts recording the viewport into a new file named after the level and the moment. */
  @ExclusiveFlow()
  public *start(): TFlow<Nullable<string>> {
    return yield* this.record();
  }

  /** Stops the recording and finishes its file, telling where it went. */
  @ExclusiveFlow()
  public *stop(): TFlow<Nullable<RecordingOutcome>> {
    return yield* this.finish();
  }

  private *record(): TFlow<Nullable<string>> {
    const level: Nullable<string> = this.snapshotService.level;

    if (!this.canStart || !level) {
      return null;
    }

    const progress: Channel<RecordingProgress> = new Channel<RecordingProgress>();
    const name: string = toLevelViewerFileStem(level, new Date());

    progress.onmessage = (told: RecordingProgress) => this.follow(told);

    const started: TCommandCompletion<Nullable<string>> = yield* this.reporter.run({
      name: "Start recording",
      subject: name,
      command: () => this.renderService.captures.startRecording(name, progress),
      failure: "Could not start recording",
    });

    if (started.result) {
      this.begin(started.result);
    }

    return started.result;
  }

  private *finish(): TFlow<Nullable<RecordingOutcome>> {
    if (!this.isRecording) {
      return null;
    }

    const stopped: TCommandCompletion<RecordingOutcome> = yield* this.reporter.run({
      name: "Stop recording",
      subject: this.file,
      command: () => levelsCommands.stopRecording(),
      failure: "Could not finish the recording",
      success: (outcome: RecordingOutcome) => ({
        title: "Recorded",
        details: `${outcome.file} · ${formatDuration((outcome.duration ?? 0) * 1000)} · ${describeDrops(outcome)}`,
        severity:
          outcome.droppedFrames + outcome.droppedSound > 0
            ? ENotificationSeverity.WARNING
            : ENotificationSeverity.SUCCESS,
      }),
    });

    this.end(stopped.result);

    return stopped.result;
  }

  /** Asks whether this machine can record. */
  private async describe(): Promise<void> {
    const availability: RecordingAvailability = await levelsCommands
      .describeRecording()
      .catch((error: unknown) => ({ isAvailable: false, reason: String(error) }));

    this.setAvailability(availability);
  }

  @BoundAction()
  private setAvailability(availability: RecordingAvailability): void {
    this.availability = availability;
  }

  @BoundAction()
  private begin(file: string): void {
    this.file = file;
    this.progress = null;
  }

  @BoundAction()
  private end(outcome: Nullable<RecordingOutcome>): void {
    this.file = null;
    this.outcome = outcome ?? this.outcome;
  }

  /** Takes what the recording told; one that ended by itself is stopped and collected, saying why. */
  @BoundAction()
  private follow(told: RecordingProgress): void {
    this.progress = told;

    if (told.ended !== null && this.isRecording) {
      this.log.warn("The recording ended by itself:", told.ended);
      void this.stop();
    }
  }
}

/**
 * @param drops - What a recording dropped so far, or once written: frames, and blocks of sound.
 * @returns What it dropped, frames and sound, or that it dropped nothing.
 */
export function describeDrops(drops: Pick<RecordingOutcome, "droppedFrames" | "droppedSound">): string {
  if (drops.droppedFrames + drops.droppedSound === 0) {
    return "nothing dropped";
  }

  return `${drops.droppedFrames} frames and ${drops.droppedSound} sound blocks dropped`;
}
