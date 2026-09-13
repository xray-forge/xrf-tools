import { inject, Injectable, OnEvent, OnProvision, WireEvent } from "@wirestate/core";
import { BoundAction, Computed, flowResult, Observable, runInAction } from "@wirestate/mobx";

import { describeTextureSaveOutcome } from "@/applications/textures-editor/lib/describe-texture-save-outcome";
import { isSameDescriptorForm, toEditableForm } from "@/applications/textures-editor/lib/texture-descriptor-form";
import { TextureEncodingService } from "@/applications/textures-editor/services/encoding";
import { texturesCommands } from "@/core/ipc/commands/textures";
import {
  EJobKind,
  TextureDescription,
  TextureDescriptorForm,
  TextureEncodingComparison,
  TextureEncodingFormat,
  TextureSaveOutcome,
  TextureVocabulary,
} from "@/core/ipc/types/xrf-app";
import { IJobNotice, IJobOutcome, IJobSettledPayload, JOB_SETTLED_EVENT } from "@/core/jobs/lib";
import { JobOperation } from "@/core/jobs/lib/job-operation";
import { JobsService } from "@/core/jobs/services/jobs";
import { getTextureIdentity } from "@/core/textures/lib/texture-identity";
import { TextureSelectionService } from "@/core/textures/services/selection";
import { AsyncState } from "@/lib/async-state";
import { Logger } from "@/lib/logging";
import { call, ExclusiveFlow, TFlow } from "@/lib/mobx";
import { Nullable } from "@/lib/types/general";

/**
 * The descriptor being edited: what is on disk, what has been typed over it, and the save that publishes the two.
 */
@Injectable()
export class TextureEditorService {
  public readonly log: Logger = new Logger(__MODULE_NAME__);

  public readonly save: JobOperation<TextureSaveOutcome>;

  /** The names the SDK gives the numbers a descriptor stores, asked for once when the editor opens. */
  @Observable()
  public vocabulary: AsyncState<TextureVocabulary> = AsyncState.idle();

  /** The form as edited, or null when no texture is selected. */
  @Observable()
  public draft: Nullable<TextureDescriptorForm> = null;

  /**
   * The source and roots the draft belongs to; labels can repeat across trees.
   */
  @Observable()
  private draftIdentity: Nullable<string> = null;

  /** The form as the backend last reported it, which is what dirtiness is measured against. */
  @Observable()
  private baseline: Nullable<TextureDescriptorForm> = null;

  /**
   * @returns Whether the descriptor form says anything the file on disk does not.
   */
  @Computed()
  public get isDescriptorDirty(): boolean {
    return !isSameDescriptorForm(this.draft, this.baseline);
  }

  /**
   * @returns Whether this node has anything pending at all, in its descriptor or in its pixels.
   */
  @Computed()
  public get isDirty(): boolean {
    return this.isDescriptorDirty || this.encodingService.isDirty;
  }

  /**
   * @returns Whether a save can be asked for: something to write, and nothing already writing.
   */
  @Computed()
  public get canSave(): boolean {
    return this.isDirty && !this.save.isRunning && this.targets !== null;
  }

  /**
   * @returns The bump mode that makes the engine bind a pair, as the backend names it.
   *
   * Zero before the vocabulary arrives, which is a value no descriptor field is ever set to from here: nothing offers
   * to generate a pair until the form it would be set from can render.
   */
  @Computed()
  public get bumpUseMode(): number {
    return this.vocabulary.value?.bumpModeUse ?? 0;
  }

  /**
   * @returns Where a save would write, or null for a texture served out of an archive.
   */
  @Computed()
  public get targets(): Nullable<TextureDescription["targets"]> {
    return this.selectionService.selected.value?.targets ?? null;
  }

  public constructor(
    private readonly selectionService: TextureSelectionService = inject(TextureSelectionService),
    private readonly encodingService: TextureEncodingService = inject(TextureEncodingService),
    jobsService: JobsService = inject(JobsService)
  ) {
    this.save = new JobOperation(jobsService, [EJobKind.TEXTURES_SAVE], this.log);
  }

  /**
   * Ask for the vocabulary the form's numeric fields are named by.
   */
  @OnProvision()
  public async onProvision(): Promise<void> {
    runInAction(() => {
      this.vocabulary = this.vocabulary.asLoading();
    });

    try {
      const vocabulary: TextureVocabulary = await texturesCommands.getVocabulary();

      runInAction(() => {
        this.vocabulary = this.vocabulary.asReady(vocabulary);
      });
    } catch (error) {
      this.log.error("Failed to read the texture vocabulary:", error);

      runInAction(() => {
        this.vocabulary = this.vocabulary.asFailed(error as Error, null);
      });
    }
  }

  /**
   * Bind the form to a described texture, keeping a draft that belongs to it.
   *
   * Called whenever a description arrives, which includes the re-describe a save triggers. A description of the texture
   * already being edited refreshes the baseline and leaves the draft alone: a save answers with what it wrote, and
   * throwing away the fields somebody is still typing into would be the wrong reading of that.
   *
   * @param description - What the backend answered, or null when nothing is selected.
   */
  @BoundAction()
  public bind(description: Nullable<TextureDescription>): void {
    const identity: Nullable<string> = description ? getTextureIdentity(description) : null;
    const form: Nullable<TextureDescriptorForm> = toEditableForm(description);

    this.baseline = form;

    if (identity !== this.draftIdentity) {
      this.encodingService.clear();
      this.draftIdentity = identity;
      this.draft = form;
    }
  }

  /**
   * Change one or more fields of the draft.
   *
   * @param patch - The fields to change and what to change them to.
   */
  @BoundAction()
  public edit(patch: Partial<TextureDescriptorForm>): void {
    if (this.draft) {
      this.draft = { ...this.draft, ...patch };
    }
  }

  /**
   * Throw the draft away and go back to what the file says.
   */
  @BoundAction()
  public discard(): void {
    this.draft = this.baseline;
    this.encodingService.clear();
  }

  /**
   * Write the draft, then re-read the texture so the panels show what landed.
   *
   * The re-describe is not optional. A save can change the descriptor's own format - writing a BC1 texture makes it
   * `tfDXT1` or `tfADXT1` by its alpha flag - and it always changes the modification stamp every target is guarded by,
   * so a second save against the stamps this one started from would be refused.
   */
  @ExclusiveFlow("save")
  public *commit(): TFlow {
    const targets: Nullable<TextureDescription["targets"]> = this.targets;
    const draft: Nullable<TextureDescriptorForm> = this.draft;

    if (!targets || !draft || !this.isDirty) {
      return;
    }

    const chosen: Nullable<TextureEncodingFormat> = this.encodingService.chosen;

    const comparison: Nullable<TextureEncodingComparison> = this.encodingService.comparison;
    const reference: Nullable<string> = this.selectionService.reference;

    if (chosen !== null && comparison === null) {
      return;
    }

    this.log.info("Saving texture descriptor:", reference);

    const { error } = yield* this.save.run({
      kind: EJobKind.TEXTURES_SAVE,
      invoke: (id: string, progress) =>
        texturesCommands.save(
          {
            // The descriptor goes every time, because writing a texture can change the format it names and the two
            // must not disagree on disk even for the moment between two writes.
            descriptor: { form: draft, target: targets.descriptor },
            texture:
              chosen === null || comparison === null
                ? null
                : { sessionId: comparison.sessionId, format: chosen, target: targets.texture },
          },
          id,
          progress
        ),
      describe: (outcome: IJobOutcome<TextureSaveOutcome>): IJobNotice =>
        describeTextureSaveOutcome(reference ?? "texture", outcome),
    });

    if (error) {
      return;
    }

    // The held comparison is priced against bytes this save has just replaced, so it is no longer an answer about
    // anything on disk.
    this.encodingService.clear();

    // Re-read, so every panel and the preview move together onto what was written.
    // Through `flowResult`, because the lane decorator hands back a cancellable rather than a generator:
    // `yield*` on it iterates nothing and the re-read silently never happens.
    yield* call(flowResult(this.selectionService.retry()));
  }

  @OnEvent(JOB_SETTLED_EVENT)
  public onJobSettled(event: WireEvent<IJobSettledPayload>): void {
    this.save.adopt(event.payload);
  }
}
