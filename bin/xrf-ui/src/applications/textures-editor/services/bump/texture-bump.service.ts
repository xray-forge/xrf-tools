import { inject, Injectable, OnEvent, WireEvent } from "@wirestate/core";
import { Computed, flowResult } from "@wirestate/mobx";

import { describeTextureBumpOutcome } from "@/applications/textures-editor/lib/describe-texture-bump-outcome";
import { toBumpReference, toBumpTarget } from "@/applications/textures-editor/lib/texture-bump-target";
import { TextureEditorService } from "@/applications/textures-editor/services/editor";
import { texturesCommands } from "@/core/ipc/commands/textures";
import { EJobKind, TextureDescription, TextureMakeBumpOutcome } from "@/core/ipc/types/xrf-app";
import { IJobNotice, IJobOutcome, IJobSettledPayload, JOB_SETTLED_EVENT } from "@/core/jobs/lib";
import { JobOperation } from "@/core/jobs/lib/job-operation";
import { JobsService } from "@/core/jobs/services/jobs";
import { TextureSelectionService } from "@/core/textures/services/selection";
import { Logger } from "@/lib/logging";
import { call, ExclusiveFlow, TFlow } from "@/lib/mobx";
import { Nullable } from "@/lib/types/general";

/** What a generation was asked to build the pair from. */
export interface ITextureBumpSources {
  /** The relief, averaged across its colour channels. Required; everything else refines it. */
  height: string;
  /** A gloss mask, or null to use one level everywhere. */
  gloss: Nullable<string>;
  /** One gloss level for the whole surface, used when there is no mask. */
  glossConstant: number;
  /** Normals to use instead of the ones the height implies, of the same size. */
  normalMap: Nullable<string>;
  /** `bump_virtual_height` of the descriptor, which decides how deep the relief is derived at. */
  virtualHeight: number;
}

/**
 * Generating the `_bump` and `_bump#` pair the open texture binds.
 *
 * Unlike a format choice, this is not pending: the pair is written the moment the run finishes, beside the texture it
 * belongs to. What it leaves pending is the descriptor - a pair on disk that no `.thm` points at binds nothing - so a
 * successful run writes the bump name and mode into the draft and leaves the node dirty for an ordinary save.
 */
@Injectable()
export class TextureBumpService {
  public readonly log: Logger = new Logger(__MODULE_NAME__);

  public readonly generate: JobOperation<TextureMakeBumpOutcome>;

  /**
   * @returns Where the pair would be written, or null when the open texture has no file on disk.
   */
  @Computed()
  public get destination(): Nullable<string> {
    return toBumpTarget(this.selectionService.selected.value);
  }

  /**
   * @returns Whether a pair can be generated for what is on screen.
   */
  @Computed()
  public get canGenerate(): boolean {
    return this.destination !== null && !this.generate.isRunning;
  }

  public constructor(
    private readonly selectionService: TextureSelectionService = inject(TextureSelectionService),
    private readonly editorService: TextureEditorService = inject(TextureEditorService),
    jobsService: JobsService = inject(JobsService)
  ) {
    this.generate = new JobOperation(jobsService, [EJobKind.TEXTURES_MAKE_BUMP], this.log);
  }

  /**
   * Build the pair, then point the descriptor at it.
   *
   * The descriptor is edited rather than written. A generation that also saved would be two acts behind one button,
   * and the fields it touches are ones a person may want to look at - or undo - before they reach disk.
   *
   * @param sources - What to build the pair from.
   */
  @ExclusiveFlow("generate")
  public *run(sources: ITextureBumpSources): TFlow {
    const description: Nullable<TextureDescription> = this.selectionService.selected.value;
    const destination: Nullable<string> = this.destination;

    if (!description || !destination) {
      return;
    }

    this.log.info("Generating bump pair for:", destination);

    const { result, error } = yield* this.generate.run({
      kind: EJobKind.TEXTURES_MAKE_BUMP,
      invoke: (id: string, progress) =>
        texturesCommands.makeBump(
          {
            destination,
            gloss: sources.gloss,
            glossConstant: sources.glossConstant,
            height: sources.height,
            // Box, because that is what the SDK's own generator leaves its parameters at.
            mipFilter: "box",
            normalMap: sources.normalMap,
            quality: "slow",
            virtualHeight: sources.virtualHeight,
          },
          id,
          progress
        ),
      describe: (outcome: IJobOutcome<TextureMakeBumpOutcome>): IJobNotice =>
        describeTextureBumpOutcome(description.reference, outcome),
    });

    if (error || result?.outcome === "cancelled") {
      return;
    }

    // The pair exists on disk and the descriptor does not name it yet, which is the one state this leaves behind.
    this.editorService.edit({
      bumpMode: this.editorService.bumpUseMode,
      bumpName: toBumpReference(description),
      virtualHeight: sources.virtualHeight,
    });

    // Through `flowResult`, because the lane decorator hands back a cancellable rather than a generator:
    // `yield*` on it iterates nothing and the re-read silently never happens.
    yield* call(flowResult(this.selectionService.retry()));
  }

  @OnEvent(JOB_SETTLED_EVENT)
  public onJobSettled(event: WireEvent<IJobSettledPayload>): void {
    this.generate.adopt(event.payload);
  }
}
