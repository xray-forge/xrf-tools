import { EventBus, inject, Injectable, OnDeactivation, OnProvision } from "@wirestate/core";
import { BoundAction, makeObservable, Observable, runInAction } from "@wirestate/mobx";

import { exportsCommands } from "@/core/bindings/commands/exports";
import { ExportSourceContent, ExportsProject } from "@/core/bindings/types/xrf-export";
import { transformError } from "@/core/error/lib";
import { releaseEditorProject } from "@/core/ipc/release";
import { emitNotification, ENotificationSeverity } from "@/core/notifications/lib";
import { EApplicationId } from "@/core/routing/application";
import { createLoadable, Loadable } from "@/lib/loadable";
import { Logger } from "@/lib/logging";
import { Nullable } from "@/lib/types/general";

@Injectable()
export class ExportsService {
  public readonly log: Logger = new Logger(__MODULE_NAME__);

  @Observable()
  public isReady: boolean = false;

  @Observable()
  public project: Loadable<Nullable<ExportsProject>> = createLoadable(null);

  public constructor(private readonly eventBus: EventBus = inject(EventBus)) {
    makeObservable(this);
  }

  @OnProvision()
  public async onProvision(): Promise<void> {
    try {
      const project: Nullable<ExportsProject> = await exportsCommands.getProject();

      if (project) {
        this.log.info("Existing exports project detected");
      } else {
        this.log.info("No existing exports project");
      }

      runInAction(() => {
        this.project = createLoadable(project);
        this.isReady = true;
      });
    } catch (error: unknown) {
      const transformed: Error = transformError(error);

      this.log.error("Failed to restore exports project:", transformed);

      runInAction(() => {
        this.project = this.project.asFailed(transformed, null);
        this.isReady = true;
      });

      emitNotification(this.eventBus, {
        details: transformed.message,
        severity: ENotificationSeverity.ERROR,
        source: EApplicationId.EXPORTS_EXPLORER,
        title: "Could not restore the open exports project",
      });
    }
  }

  /** Releases parsed exports when the editor deactivates. */
  @OnDeactivation()
  public onDeactivation(): void {
    releaseEditorProject(exportsCommands.closeProject);
  }

  /**
   * Reads the source that declares an extern.
   *
   * @param name - Declaration name reported by the project.
   * @returns Resolves to the source content that declares the extern.
   */
  @BoundAction()
  public async readExportSource(name: string): Promise<ExportSourceContent> {
    this.log.info("Reading export source:", name);

    return exportsCommands.getSource(name);
  }

  @BoundAction()
  public async openExportsProject(path: string): Promise<void> {
    if (this.project.isLoading) {
      return this.log.info("Skip parsing exports while another operation is running:", path);
    }

    this.log.info("Parsing exports from project:", path);
    this.project = this.project.asLoading(null);

    try {
      const result: ExportsProject = await exportsCommands.openProject(path);

      runInAction(() => (this.project = this.project.asReady(result)));
    } catch (error: unknown) {
      const transformed: Error = transformError(error);

      this.log.error("Failed to parse exports:", transformed);
      runInAction(() => (this.project = this.project.asFailed(transformed, null)));

      emitNotification(this.eventBus, {
        details: `${path}\n${transformed.message}`,
        severity: ENotificationSeverity.ERROR,
        source: EApplicationId.EXPORTS_EXPLORER,
        title: "Could not parse exports",
      });
    }
  }

  @BoundAction()
  public async refreshExportsProject(): Promise<void> {
    const existing: Nullable<ExportsProject> = this.project.value;

    if (!existing || this.project.isLoading) {
      return;
    }

    this.log.info("Refreshing exports project:", existing.root);
    this.project = this.project.asLoading(existing);

    try {
      const result: ExportsProject = await exportsCommands.openProject(existing.root);

      runInAction(() => (this.project = this.project.asReady(result)));
    } catch (error: unknown) {
      const transformed: Error = transformError(error);

      this.log.error("Failed to refresh exports project:", transformed);
      runInAction(() => (this.project = this.project.asFailed(transformed, existing)));

      emitNotification(this.eventBus, {
        details: `${existing.root}\n${transformed.message}`,
        severity: ENotificationSeverity.ERROR,
        source: EApplicationId.EXPORTS_EXPLORER,
        title: "Could not refresh exports",
      });
    }
  }

  @BoundAction()
  public async closeExportsProject(): Promise<void> {
    const existing: Nullable<ExportsProject> = this.project.value;

    if (this.project.isLoading) {
      return;
    }

    this.log.info("Closing exports project");
    this.project = this.project.asLoading(existing);

    try {
      await exportsCommands.closeProject();
      // Cleared on purpose: closing swaps the viewer for the application's picker in place. It used to
      // hold the project until the caller navigated away, because clearing it unmounted the editor
      // before React Router could process that navigation. Nothing navigates on close any more.
      runInAction(() => (this.project = this.project.asReady(null)));
    } catch (error: unknown) {
      const transformed: Error = transformError(error);

      this.log.error("Failed to close exports project:", transformed);
      runInAction(() => (this.project = this.project.asReady(existing)));

      emitNotification(this.eventBus, {
        details: transformed.message,
        severity: ENotificationSeverity.ERROR,
        source: EApplicationId.EXPORTS_EXPLORER,
        title: "Could not close exports project",
      });

      throw transformed;
    }
  }
}
