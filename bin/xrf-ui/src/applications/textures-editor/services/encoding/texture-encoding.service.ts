import { inject, Injectable, OnDeactivation, OnEvent, WireEvent } from "@wirestate/core";
import { BoundAction, Computed, Observable } from "@wirestate/mobx";

import { describeTextureCompareOutcome } from "@/applications/textures-editor/lib/describe-texture-compare-outcome";
import { texturesCommands } from "@/core/bindings/commands/textures";
import { texturesRawCommands } from "@/core/bindings/commands/textures-raw";
import {
  EJobKind,
  SessionId,
  TextureDescription,
  TextureEncodingComparison,
  TextureEncodingFormat,
  TextureEncodingReport,
} from "@/core/bindings/types/xrf-app";
import { transformError } from "@/core/error/lib";
import { Session } from "@/core/ipc/session";
import { IJobNotice, IJobOutcome, IJobSettledPayload, JOB_SETTLED_EVENT } from "@/core/jobs/lib";
import { JobOperation } from "@/core/jobs/lib/job-operation";
import { JobsService } from "@/core/jobs/services/jobs";
import { getTextureIdentity } from "@/core/textures/lib/texture-identity";
import { TextureSelectionService } from "@/core/textures/services/selection";
import { AsyncState } from "@/lib/async-state";
import { Logger } from "@/lib/logging";
import { call, cancelFlow, ExclusiveFlow, LatestFlow, TFlow } from "@/lib/mobx";
import { Nullable } from "@/lib/types/general";

/**
 * What every format would cost this texture, and which one the editor is holding for a save.
 */
@Injectable()
export class TextureEncodingService {
  public readonly log: Logger = new Logger(__MODULE_NAME__);

  private readonly session: Session = new Session(texturesCommands.close);

  public readonly compare: JobOperation<TextureEncodingComparison>;

  /**
   * The candidate chosen to replace the base texture, or null when the file on disk still stands.
   */
  @Observable()
  private choice: Nullable<{ sessionId: SessionId; format: TextureEncodingFormat }> = null;

  @Computed()
  public get chosen(): Nullable<TextureEncodingFormat> {
    return this.choice?.sessionId === this.comparison?.sessionId ? (this.choice?.format ?? null) : null;
  }

  /**
   * The chosen candidate as a picture, for showing it beside the texture it would replace.
   */
  @Observable()
  public preview: AsyncState<ArrayBuffer> = AsyncState.idle();

  /**
   * @returns The comparison to show, or null when none belongs to the texture on screen.
   */
  @Computed()
  public get comparison(): Nullable<TextureEncodingComparison> {
    const selected: Nullable<TextureDescription> = this.selectionService.selected.value;
    const comparison: Nullable<TextureEncodingComparison> = this.compare.result;

    return selected && comparison && getTextureIdentity(selected) === getTextureIdentity(comparison)
      ? comparison
      : null;
  }

  /**
   * @returns Whether a candidate is being held for the save.
   */
  @Computed()
  public get isDirty(): boolean {
    return this.chosen !== null;
  }

  /**
   * @returns What the chosen candidate cost, for a surface stating what a save would write.
   */
  @Computed()
  public get chosenReport(): Nullable<TextureEncodingReport> {
    const chosen: Nullable<TextureEncodingFormat> = this.chosen;

    return this.comparison?.candidates.find((candidate) => candidate.format === chosen) ?? null;
  }

  public constructor(
    private readonly selectionService: TextureSelectionService = inject(TextureSelectionService),
    jobsService: JobsService = inject(JobsService)
  ) {
    this.compare = new JobOperation(jobsService, [EJobKind.TEXTURES_COMPARE_ENCODINGS], this.log);
  }

  @OnDeactivation()
  public onDeactivation(): void {
    this.clear();
  }

  /**
   * Forget the comparison and the choice, for a texture that is no longer the one on screen.
   */
  @BoundAction()
  public clear(): void {
    this.session.release();

    this.choice = null;
    this.preview = this.preview.asIdle();

    cancelFlow(this, "preview");

    this.compare.reset();
  }

  /**
   * Choose a candidate to write, or clear the choice by naming the one already chosen, and read its picture.
   *
   * Superseding rather than exclusive: clicking down a list of candidates is an ordinary way to look at them, and
   * only the last one asked for is the one anybody is waiting to see.
   *
   * @param format - The candidate to hold, or the held one to release.
   */
  @LatestFlow("preview")
  public *choose(format: TextureEncodingFormat): TFlow {
    const comparison: Nullable<TextureEncodingComparison> = this.comparison;

    if (
      !comparison ||
      this.compare.isRunning ||
      !comparison.candidates.some((candidate) => candidate.format === format)
    ) {
      return;
    }

    // Before the first yield, so a caller that reads the choice straight after asking for it sees the answer.
    const chosen: Nullable<TextureEncodingFormat> = this.chosen === format ? null : format;

    this.choice = chosen === null ? null : { sessionId: comparison.sessionId, format: chosen };

    if (chosen === null) {
      this.preview = this.preview.asIdle();

      return;
    }

    this.preview = this.preview.asLoading(null);

    try {
      const bytes: ArrayBuffer = yield* call(texturesRawCommands.readCandidate(comparison.sessionId, chosen));

      if (this.comparison?.sessionId === comparison.sessionId) {
        this.preview = this.preview.asReady(bytes);
      }
    } catch (error: unknown) {
      const transformed: Error = transformError(error);

      this.log.error("Failed to decode the chosen candidate:", transformed);

      if (this.comparison?.sessionId === comparison.sessionId) {
        this.preview = this.preview.asFailed(transformed, null);
      }
    }
  }

  /**
   * Weigh every candidate format against the texture on screen.
   *
   * Chosen candidates do not survive it. The backend keeps one comparison at a time and this replaces it, so a name
   * held from the previous one would address bytes that are gone.
   *
   * @param mipFilter - Kernel the chain is reduced with, by its SDK name, or null to weigh the base level alone.
   */
  @ExclusiveFlow("compare")
  public *run(mipFilter: Nullable<string>): TFlow {
    const description: Nullable<TextureDescription> = this.selectionService.selected.value;

    if (!description || this.compare.isRunning) {
      return;
    }

    this.choice = null;
    cancelFlow(this, "preview");
    this.preview = this.preview.asIdle();

    this.log.info("Comparing texture encodings:", description.reference);

    yield* this.compare.run({
      kind: EJobKind.TEXTURES_COMPARE_ENCODINGS,
      invoke: (id: string, progress) =>
        this.session.open((sessionId) =>
          texturesCommands.compareEncodings(
            { sessionId, mipFilter, quality: "slow", roots: description.roots, source: description.source },
            id,
            progress
          )
        ),
      describe: (outcome: IJobOutcome<TextureEncodingComparison>): IJobNotice =>
        describeTextureCompareOutcome(description.reference, outcome),
    });
  }

  @OnEvent(JOB_SETTLED_EVENT)
  public onJobSettled(event: WireEvent<IJobSettledPayload>): void {
    this.compare.adopt(event.payload);
    this.session.adopt(this.compare.result);
  }
}
