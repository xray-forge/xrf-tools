import { Channel } from "@tauri-apps/api/core";
import { inject, Injectable, OnActivation, OnDeactivation } from "@wirestate/core";
import { BoundAction, Computed, RefObservable } from "@wirestate/mobx";
import { ExclusiveFlow, TFlow } from "@xrf/flow";
import { levelsCommands } from "@xrf/ipc/commands/levels";
import { systemCommands } from "@xrf/ipc/commands/system";
import { RecordingAvailability, RecordingOutcome, RecordingProgress } from "@xrf/ipc/types/xrf-app-levels";
import { Logger } from "@xrf/logging";
import { Nullable } from "@xrf/types";

import { KeybindCommand } from "@/core/commands";
import { LevelRenderService } from "@/core/level/services/level-render";
import { ENotificationSeverity } from "@/core/notifications/lib";
import { CommandReporter, TCommandCompletion } from "@/core/operations/lib/command-reporter";
import { RECORD_LEVEL_KEYBIND_COMMAND } from "@/tools/level-viewer/commands";
import { toLevelViewerFileStem } from "@/tools/level-viewer/lib/files/level-viewer-file-stem";
import { LevelViewerRecordingReadout } from "@/tools/level-viewer/lib/recording/level-viewer-recording-readout";
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

  /**
   * Starts a recording where none runs, else stops the one running. It shares one lane with {@link start} and
   * {@link stop}: the backend keeps one recording, so a start or stop asked while another is under way joins it.
   */
  @KeybindCommand(RECORD_LEVEL_KEYBIND_COMMAND, {
    isEnabled: (service: LevelViewerRecordingService) => service.isRecording || service.canStart,
  })
  @ExclusiveFlow<LevelViewerRecordingService>("file")
  public *toggle(): TFlow {
    if (this.isRecording) {
      yield* this.finish();
    } else {
      yield* this.record();
    }
  }

  /** Starts recording the viewport into a new file named after the level and the moment. */
  @ExclusiveFlow<LevelViewerRecordingService>("file")
  public *start(): TFlow<Nullable<string>> {
    return yield* this.record();
  }

  /** Stops the recording and finishes its file, its toast telling where it went. */
  @ExclusiveFlow<LevelViewerRecordingService>("file")
  public *stop(): TFlow<Nullable<RecordingOutcome>> {
    return yield* this.finish();
  }

  /** Deletes the last recording for good. */
  @ExclusiveFlow()
  public *remove(): TFlow {
    const outcome: Nullable<RecordingOutcome> = this.outcome;

    if (!outcome) {
      return;
    }

    const completion: TCommandCompletion<unknown> = yield* this.reporter.run({
      name: "Delete recording",
      subject: outcome.file,
      command: () => levelsCommands.deleteRecording(outcome.file),
      failure: "Could not delete the recording",
    });

    if (!completion.error) {
      this.forget();
    }
  }

  /** Shows the last recording selected in its folder. */
  @ExclusiveFlow()
  public *reveal(): TFlow {
    const outcome: Nullable<RecordingOutcome> = this.outcome;

    if (outcome) {
      yield* this.reporter.run({
        name: "Show recording",
        subject: outcome.file,
        command: () => systemCommands.revealPath(outcome.file),
        failure: "Could not show the recording",
      });
    }
  }

  /** Puts the last recording's path on the clipboard. */
  @ExclusiveFlow()
  public *copyPath(): TFlow {
    const outcome: Nullable<RecordingOutcome> = this.outcome;

    if (outcome) {
      yield* this.reporter.run({
        name: "Copy recording path",
        subject: outcome.file,
        command: () => navigator.clipboard.writeText(outcome.file),
        failure: "Could not copy the recording's path",
      });
    }
  }

  private *record(): TFlow<Nullable<string>> {
    const level: Nullable<string> = this.snapshotService.level;

    if (!this.canStart || !level) {
      return null;
    }

    const progress: Channel<RecordingProgress> = new Channel<RecordingProgress>();
    const name: string = toLevelViewerFileStem(level, new Date());

    progress.onmessage = (told: RecordingProgress) => this.follow(told);
    this.prepare();

    const started: TCommandCompletion<Nullable<string>> = yield* this.reporter.run({
      name: "Start recording",
      subject: name,
      command: () => this.renderService.captures.startRecording(name, progress),
      failure: "Could not start recording",
    });

    if (started.result) {
      this.begin(started.result);

      // A recording the renderer refused at once, as while another long job runs, can say it ended before its start
      // answers; it is collected here, its failure telling why.
      if (this.progress?.ended) {
        yield* this.finish();
      }
    }

    return this.file;
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
      // Its toast tells it; a notice stays only where it dropped something, which the toast's fading would lose.
      success: (outcome: RecordingOutcome) =>
        LevelViewerRecordingReadout.hasDrops(outcome)
          ? {
              title: "Recorded with drops",
              details: `${outcome.file} · ${LevelViewerRecordingReadout.describeOutcome(outcome)}`,
              severity: ENotificationSeverity.WARNING,
            }
          : null,
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
  private prepare(): void {
    this.progress = null;
  }

  @BoundAction()
  private begin(file: string): void {
    this.file = file;
  }

  @BoundAction()
  private end(outcome: Nullable<RecordingOutcome>): void {
    this.file = null;
    this.outcome = outcome ?? this.outcome;
  }

  @BoundAction()
  private forget(): void {
    this.outcome = null;
  }

  /**
   * Takes what the recording told; one that ended by itself is stopped and collected, saying why. One that ends while
   * it starts is collected by the start itself, since this stop joins the start's lane.
   */
  @BoundAction()
  private follow(told: RecordingProgress): void {
    this.progress = told;

    if (told.ended !== null && this.isRecording) {
      this.log.warn("The recording ended by itself:", told.ended);
      void this.stop();
    }
  }
}
