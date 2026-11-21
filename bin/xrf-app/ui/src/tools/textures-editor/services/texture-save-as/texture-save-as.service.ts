import { inject, Injectable } from "@wirestate/core";
import { Computed, flowResult, Observable } from "@wirestate/mobx";
import { AsyncState } from "@xrf/async";
import { transformError } from "@xrf/errors";
import { call, ExclusiveFlow, LatestFlow, TFlow } from "@xrf/flow";
import { texturesCommands } from "@xrf/ipc/commands/textures";
import {
  ETextureSource,
  TextureDescription,
  TextureDescriptorForm,
  TextureEncodingComparison,
  TEXTURES_JOB_SAVE_AS,
  TextureSaveAsBump,
  TextureSaveAsOutcome,
  TextureSaveAsPlan,
  TextureSaveAsTarget,
} from "@xrf/ipc/types/xrf-app-textures";
import { EJobOutcome } from "@xrf/ipc/types/xrf-job";
import { TextureEncodingFormat } from "@xrf/ipc/types/xrf-texture";
import { Logger } from "@xrf/logging";
import { Nullable } from "@xrf/types";

import { IJobNotice, IJobOutcome } from "@/core/jobs/lib";
import { JobOperation } from "@/core/jobs/lib/job-operation";
import { TCommandCompletion } from "@/core/operations/lib/command-reporter";
import { TextureSelectionService } from "@/core/textures/services/texture-selection";
import { describeTextureSaveAsOutcome } from "@/tools/textures-editor/lib/describe-texture-save-as-outcome";
import { TextureEditorService } from "@/tools/textures-editor/services/texture-editor";
import { TextureEncodingService } from "@/tools/textures-editor/services/texture-encoding";

/** The choices a Save as is made with. */
export interface ITextureSaveAs {
  /** Texture name for the copy, such as `ston\ston_beton05_copy`, in the roots the original was opened from. */
  destination: string;
  /** Whether the copy gets its own bump map or shares the original's. */
  bump: TextureSaveAsBump;
}

/**
 * Saves the open texture, as edited, under a new name, leaving the original unchanged, then opens the copy.
 *
 * The backend plans a copy before writing it, and the dialog shows that plan, so the files and fields a person agrees
 * to are the ones written.
 */
@Injectable()
export class TextureSaveAsService {
  public readonly log: Logger = new Logger(__MODULE_NAME__);

  public readonly saveAs: JobOperation<TextureSaveAsOutcome> = new JobOperation(this.log, {
    kinds: [TEXTURES_JOB_SAVE_AS],
  });

  /** What a copy under the last checked name would write. */
  @Observable()
  public plan: AsyncState<TextureSaveAsPlan> = AsyncState.idle();

  /**
   * @returns Whether Save as is available: a loose texture opened from its roots, and no copy being written.
   */
  @Computed()
  public get canSaveAs(): boolean {
    return (
      // A file opened from outside every root has no texture name for a copy; it is described without a material.
      Boolean(this.selectionService.selected.value?.targets && this.selectionService.selected.value.material) &&
      this.editorService.draft !== null &&
      !this.saveAs.isRunning
    );
  }

  public constructor(
    private readonly selectionService: TextureSelectionService = inject(TextureSelectionService),
    private readonly editorService: TextureEditorService = inject(TextureEditorService),
    private readonly encodingService: TextureEncodingService = inject(TextureEncodingService)
  ) {}

  /**
   * Check a name for the copy and ask what the copy would write, writing nothing.
   *
   * The previous plan stays available while the next loads or when the name is refused, so the dialog keeps its size.
   *
   * @param request - The copy's name, and what it does with the bump map.
   */
  @LatestFlow("plan")
  public *review(request: ITextureSaveAs): TFlow {
    const { destination, bump } = request;
    const target: Nullable<TextureSaveAsTarget> = this.toTarget(destination);
    const draft: Nullable<TextureDescriptorForm> = this.editorService.draft;

    if (!target || !draft) {
      this.plan = this.plan.asIdle();

      return;
    }

    if (!destination.trim()) {
      this.plan = this.plan.asFailed(new Error("Enter a name for the copy."));

      return;
    }

    this.plan = this.plan.asLoading();

    try {
      const plan: TextureSaveAsPlan = yield* call(texturesCommands.planSaveAs({ target, form: draft, bump }));

      this.plan = this.plan.asReady(plan);
    } catch (error: unknown) {
      this.plan = this.plan.asFailed(transformError(error));
    }
  }

  /**
   * Write the copy, then open it.
   *
   * @param request - The copy's name, and what it does with the bump map.
   * @param isForced - Whether to replace files that already have the copy's names; true only after the person agreed.
   * @returns How the job ended, or null when it did not start.
   */
  @ExclusiveFlow("saveAs")
  public *run(request: ITextureSaveAs, isForced: boolean): TFlow<Nullable<TCommandCompletion<TextureSaveAsOutcome>>> {
    const description: Nullable<TextureDescription> = this.selectionService.selected.value;
    const target: Nullable<TextureSaveAsTarget> = this.toTarget(request.destination);
    const draft: Nullable<TextureDescriptorForm> = this.editorService.draft;
    const chosen: Nullable<TextureEncodingFormat> = this.encodingService.chosen;
    const comparison: Nullable<TextureEncodingComparison> = this.encodingService.comparison;

    if (!description?.targets || !target || !draft || (chosen !== null && comparison === null)) {
      return null;
    }

    const sources: NonNullable<TextureDescription["targets"]> = description.targets;

    this.log.info("Saving texture as:", target.reference, "->", target.destination);

    const completion: Nullable<TCommandCompletion<TextureSaveAsOutcome>> = yield* this.saveAs.run({
      kind: TEXTURES_JOB_SAVE_AS,
      invoke: (id: string, progress) =>
        texturesCommands.saveAs(
          {
            target,
            form: draft,
            sources,
            texture:
              chosen === null || comparison === null
                ? null
                : { sessionId: comparison.sessionId, format: chosen, expected: sources.texture.expected },
            bump: request.bump,
            isForced,
          },
          id,
          progress
        ),
      describe: (outcome: IJobOutcome<TextureSaveAsOutcome>): IJobNotice =>
        describeTextureSaveAsOutcome(target.reference, target.destination, outcome),
    });

    if (!completion || completion.error || completion.result.outcome === EJobOutcome.CANCELLED) {
      return completion;
    }

    this.plan = this.plan.asIdle();

    // The copy becomes the open texture, with nothing pending; the original stays as it is on disk.
    // Through `flowResult`, because the lane decorator hands back a cancellable rather than a generator.
    yield* call(
      flowResult(
        this.selectionService.open(
          { kind: ETextureSource.ASSET, reference: completion.result.reference },
          description.roots
        )
      )
    );

    return completion;
  }

  /**
   * @param destination - Texture name for the copy.
   * @returns The open texture and that name, or null when no loose texture is open.
   */
  private toTarget(destination: string): Nullable<TextureSaveAsTarget> {
    const description: Nullable<TextureDescription> = this.selectionService.selected.value;

    if (!description?.targets) {
      return null;
    }

    return { roots: description.roots, reference: description.reference, destination };
  }
}
