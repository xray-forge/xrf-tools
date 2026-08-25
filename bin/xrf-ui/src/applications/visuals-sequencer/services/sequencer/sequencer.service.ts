import { EventBus, inject, Injectable, OnDeactivation, OnProvision } from "@wirestate/core";
import { BoundAction, Computed, flowResult, Observable, runInAction } from "@wirestate/mobx";
import { Texture } from "three";

import { VisualSequenceService } from "@/applications/visuals-sequencer/services/sequence";
import { createRoots } from "@/core/assets/lib";
import { visualsCommands } from "@/core/bindings/commands/visuals";
import { SelectedVisualDescription, VisualSource } from "@/core/bindings/types/xrf-app";
import { XrayRoots } from "@/core/bindings/types/xrf-vfs";
import { transformError } from "@/core/error/lib";
import { releaseEditorProject } from "@/core/ipc/release";
import { emitNotification, ENotificationSeverity } from "@/core/notifications/lib";
import { EApplicationId } from "@/core/routing/application";
import { getProjectGamedataPath } from "@/core/settings/lib/path/project";
import { ProjectService } from "@/core/settings/services/project/project.service";
import { describeVisualSource } from "@/core/visuals/lib/visual-source";
import { IOpenVisual, VisualLoadService } from "@/core/visuals/services/visual-load.service";
import { createLoadable, Loadable } from "@/lib/loadable";
import { Logger } from "@/lib/logging";
import { Nullable } from "@/lib/types/general";

/**
 * The visual a sequence is being written against, and the motions it can be written out of.
 */
@Injectable()
export class SequencerService {
  public readonly log: Logger = new Logger(__MODULE_NAME__);

  @Observable()
  public isReady: boolean = false;

  /** Every motion the open visual can play, which is what clips are picked from. */
  @Observable()
  public motions: Loadable<Array<string>> = createLoadable([]);

  /**
   * @returns The visual being shown, straight from the loader.
   */
  @Computed()
  public get visual(): Loadable<Nullable<IOpenVisual>> {
    return this.loadService.visual;
  }

  @Computed()
  public get textures(): ReadonlyMap<number, Texture> {
    return this.loadService.textures;
  }

  @Computed()
  public get sourceLabel(): Nullable<string> {
    return this.loadService.sourceLabel;
  }

  @Computed()
  public get hasMotions(): boolean {
    return this.loadService.hasMotions;
  }

  public constructor(
    private readonly eventBus: EventBus = inject(EventBus),
    private readonly projectService: ProjectService = inject(ProjectService),
    private readonly loadService: VisualLoadService = inject(VisualLoadService),
    private readonly sequenceService: VisualSequenceService = inject(VisualSequenceService)
  ) {}

  /**
   * Restore whatever the backend still has selected.
   */
  @OnProvision()
  public async onProvision(): Promise<void> {
    try {
      const selected: Nullable<SelectedVisualDescription> = await visualsCommands.getModel();

      if (selected) {
        this.log.info("Restoring selected visual:", describeVisualSource(selected.source));

        await flowResult(this.loadService.restore(selected));
        await this.list();
      }
    } catch (error) {
      this.log.error("Failed to restore selected visual:", error);
    } finally {
      runInAction(() => {
        this.isReady = true;
      });
    }
  }

  @OnDeactivation()
  public onDeactivation(): void {
    this.log.info("Deactivating and disposing the sequencer");

    this.sequenceService.clear();
    this.loadService.clear();
    releaseEditorProject(visualsCommands.closeModel);
  }

  /**
   * Open a loose visual from disk and list what it can play.
   *
   * @param path - Filesystem path of the `.ogf` file.
   */
  @BoundAction()
  public async openFile(path: string): Promise<void> {
    // Centred on the file, so its own tree and installation are searched for its textures and its animation banks.
    await this.open({ kind: "file", path }, path);
  }

  /** Close what is open, on screen and in the backend. */
  @BoundAction()
  public async close(): Promise<void> {
    this.sequenceService.clear();
    this.loadService.clear();

    runInAction(() => {
      this.motions = createLoadable([]);
    });

    try {
      await visualsCommands.closeModel();
    } catch (error) {
      this.log.error("Failed to close visual:", error);
    }
  }

  /**
   * Load a visual and report a failure the way this application reports one.
   *
   * @param source - Visual source to open.
   * @param asset - Asset the roots are centred on, whose own tree is searched first.
   */
  private async open(source: VisualSource, asset: Nullable<string> = null): Promise<void> {
    // A track names motions of the model it was written against, and the backend is about to park a different one.
    this.sequenceService.clear();

    runInAction(() => {
      this.motions = createLoadable([]);
    });

    await this.loadService.load(source, await this.getRoots(asset));

    const error: Nullable<Error> = this.visual.error;

    if (error) {
      emitNotification(this.eventBus, {
        details: `${describeVisualSource(source)}\n${transformError(error).message}`,
        severity: ENotificationSeverity.ERROR,
        source: EApplicationId.VISUALS_SEQUENCER,
        title: "Could not open visual",
      });

      return;
    }

    await this.list();
  }

  /**
   * Names what the open visual can play.
   */
  private async list(): Promise<void> {
    if (!this.hasMotions) {
      return;
    }

    runInAction(() => {
      this.motions = this.motions.asLoading();
    });

    try {
      const names: Array<string> = await visualsCommands.listMotions();

      runInAction(() => {
        this.motions = this.motions.asReady(names);
      });
    } catch (error: unknown) {
      const transformed: Error = transformError(error);

      this.log.error("Failed to list motions:", transformed);

      runInAction(() => {
        this.motions = this.motions.asFailed(transformed, []);
      });
    }
  }

  /**
   * The roots a visual's references are searched in, after the visual's own tree.
   *
   * @param asset - Asset the roots are centred on.
   * @returns The roots spec to open with.
   */
  private async getRoots(asset: Nullable<string> = null): Promise<XrayRoots> {
    const projectPath: Nullable<string> = this.projectService.xrfProjectPath;
    const project: Array<string> = projectPath ? [await getProjectGamedataPath(projectPath)] : [];

    return createRoots(project, asset);
  }
}
