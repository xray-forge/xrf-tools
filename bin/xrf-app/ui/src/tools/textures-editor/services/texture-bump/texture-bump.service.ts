import { inject, Injectable } from "@wirestate/core";
import { Computed, flowResult } from "@wirestate/mobx";
import { call, ExclusiveFlow, TFlow } from "@xrf/flow";
import { texturesCommands } from "@xrf/ipc/commands/textures";
import {
  TextureBumpGloss,
  TextureDescription,
  TextureMakeBumpOutcome,
  TEXTURES_JOB_MAKE_BUMP,
} from "@xrf/ipc/types/xrf-app-textures";
import { EJobOutcome } from "@xrf/ipc/types/xrf-job";
import { Logger } from "@xrf/logging";
import { Nullable } from "@xrf/types";

import { IJobNotice, IJobOutcome } from "@/core/jobs/lib";
import { JobOperation } from "@/core/jobs/lib/job-operation";
import { TCommandCompletion } from "@/core/operations/lib/command-reporter";
import { TextureSelectionService } from "@/core/textures/services/texture-selection";
import { describeTextureBumpOutcome } from "@/tools/textures-editor/lib/describe-texture-bump-outcome";
import { toBumpTarget } from "@/tools/textures-editor/lib/texture-bump-target";
import { TextureEditorService } from "@/tools/textures-editor/services/texture-editor";

/** What a generation was asked to build the pair from. */
export interface ITextureBumpSources {
  /** The relief, averaged across its colour channels. Required; everything else refines it. */
  height: string;
  /** Where the gloss comes from: the height map's own alpha, as the SDK reads it, a mask, or one level everywhere. */
  gloss: TextureBumpGloss;
  /** Normals to use instead of the ones the height implies, of the same size. */
  normalMap: Nullable<string>;
  /** `bump_virtual_height` of the descriptor, which decides how deep the relief is derived at; null for the SDK's. */
  virtualHeight: Nullable<number>;
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

  public readonly generate: JobOperation<TextureMakeBumpOutcome> = new JobOperation(this.log, {
    kinds: [TEXTURES_JOB_MAKE_BUMP],
  });

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
    private readonly editorService: TextureEditorService = inject(TextureEditorService)
  ) {}

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

    const completion: Nullable<TCommandCompletion<TextureMakeBumpOutcome>> = yield* this.generate.run({
      kind: TEXTURES_JOB_MAKE_BUMP,
      invoke: (id: string, progress) =>
        texturesCommands.makeBump(
          {
            destination,
            gloss: sources.gloss,
            height: sources.height,
            // The backend's own chain, the Box the SDK's generator leaves its parameters at.
            mipFilter: null,
            normalMap: sources.normalMap,
            quality: "slow",
            virtualHeight: sources.virtualHeight,
          },
          id,
          progress
        ),
      describe: (outcome: IJobOutcome<TextureMakeBumpOutcome>): IJobNotice =>
        describeTextureBumpOutcome(
          description.reference,
          outcome,
          this.editorService.vocabulary.value?.minimumGlossPower ?? null
        ),
    });

    if (!completion || completion.error || completion.result?.outcome === EJobOutcome.CANCELLED) {
      return;
    }

    // The pair exists on disk and the descriptor does not name it yet, which is the one state this leaves behind.
    this.editorService.edit({
      bumpMode: this.editorService.bumpUseMode,
      bumpName: description.generatedBump.bump,
      virtualHeight: sources.virtualHeight,
    });

    // Through `flowResult`, because the lane decorator hands back a cancellable rather than a generator:
    // `yield*` on it iterates nothing and the re-read silently never happens.
    yield* call(flowResult(this.selectionService.retry()));
  }
}
