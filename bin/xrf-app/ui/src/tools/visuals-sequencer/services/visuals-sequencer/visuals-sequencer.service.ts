import { inject, Injectable, OnDeactivation, OnProvision } from "@wirestate/core";
import { BoundAction, Computed, flowResult } from "@wirestate/mobx";
import { cancelFlow, LatestFlow, TFlow } from "@xrf/flow";
import { visualsCommands } from "@xrf/ipc/commands/visuals";
import { AssetSource, EAssetSource } from "@xrf/ipc/types/xrf-app-core";
import { XrayRoots } from "@xrf/ipc/types/xrf-vfs";
import { Logger } from "@xrf/logging";
import { Nullable } from "@xrf/types";

import { createRoots } from "@/core/assets/lib";
import { CommandOperation } from "@/core/operations/lib/command-operation";
import { IVisualPose } from "@/core/visuals/lib/render";
import { VisualInspectionService } from "@/core/visuals/services/visual-inspection";
import { VisualLoadService } from "@/core/visuals/services/visual-load";
import { VisualSequenceService } from "@/tools/visuals-sequencer/services/visual-sequence";

/**
 * The visual a sequence is being written against, and the motions it can be written out of.
 */
@Injectable()
export class VisualsSequencerService extends VisualInspectionService {
  public readonly log: Logger = new Logger(__MODULE_NAME__);

  /** Every motion the open visual can play, which is what clips are picked from. */
  public readonly motions: CommandOperation<Array<string>> = new CommandOperation(this.log);

  /**
   * @returns How the model stands, which here is the track rather than any one picked motion.
   */
  @Computed()
  public get pose(): IVisualPose {
    return { frame: this.sequenceService.frame, motion: this.sequenceService.motion };
  }

  /**
   * @returns Nothing: a sequencer composes motions rather than inspecting a skeleton, so its bone panel reads.
   *
   * Marking and hiding are the viewer's, and a track that recorded hidden bones would be a sequence document decision
   * rather than a panel one.
   */
  public get boneControls(): null {
    return null;
  }

  public constructor(
    loadService: VisualLoadService = inject(VisualLoadService),
    private readonly sequenceService: VisualSequenceService = inject(VisualSequenceService)
  ) {
    super(loadService);
  }

  /**
   * Restore whatever the backend still has selected.
   */
  @OnProvision()
  public async onProvision(): Promise<void> {
    await this.loadService.restore();
    await flowResult(this.list());
  }

  @OnDeactivation()
  public onDeactivation(): void {
    this.sequenceService.clear();
  }

  /**
   * Open a loose visual from disk and list what it can play.
   *
   * @param path - Filesystem path of the `.ogf` file.
   * @param assetRoot - A further tree its references are searched in, or null to search only its own.
   */
  @BoundAction()
  public async openFile(path: string, assetRoot: Nullable<string> = null): Promise<void> {
    // Centred on the file, so its own tree is searched for its textures and its animation banks, with the named root
    // behind it.
    await this.open({ kind: EAssetSource.FILE, path }, path, assetRoot);
  }

  /** Close what is open, on screen and in the backend. */
  @BoundAction()
  public async close(): Promise<void> {
    this.forgetMotions();

    await this.loadService.close();
  }

  /**
   * Load a visual and list what it can play.
   *
   * @param source - Visual source to open.
   * @param asset - Asset the roots are centred on, whose own tree is searched first.
   * @param assetRoot - A further tree searched behind it.
   */
  private async open(
    source: AssetSource,
    asset: Nullable<string> = null,
    assetRoot: Nullable<string> = null
  ): Promise<void> {
    this.forgetMotions();

    if (await this.loadService.load(source, await this.getRoots(asset, assetRoot))) {
      await flowResult(this.list());
    }
  }

  /** Lets the motions of the visual held go, with the track written against them. */
  @BoundAction()
  private forgetMotions(): void {
    // A track names motions of the model it was written against, and the backend is about to park a different one.
    this.sequenceService.clear();
    cancelFlow(this, "motions");
    this.motions.reset();
  }

  /**
   * Names what the open visual can play; a newer listing, for the visual opened since, replaces this one.
   */
  @LatestFlow("motions")
  private *list(): TFlow {
    const sessionId: Nullable<string> = this.sessionId;

    if (!this.hasMotions || !sessionId) {
      return;
    }

    yield* this.motions.run({
      name: "List sequencer motions",
      subject: sessionId,
      command: () => visualsCommands.listMotions(sessionId).then((listed) => listed.value),
      failure: "Could not list the motions",
    });
  }

  /**
   * The roots a visual's references are searched in, after the visual's own tree.
   *
   * @param asset - Asset the roots are centred on.
   * @param assetRoot - A further tree searched behind it.
   * @returns The roots spec to open with.
   */
  private async getRoots(asset: Nullable<string> = null, assetRoot: Nullable<string> = null): Promise<XrayRoots> {
    return createRoots([assetRoot], asset);
  }
}
