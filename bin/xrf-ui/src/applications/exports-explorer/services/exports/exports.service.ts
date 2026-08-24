import { EventBus, inject, Injectable, OnDeactivation, OnProvision } from "@wirestate/core";
import { BoundAction, flowResult, Observable } from "@wirestate/mobx";

import { exportsCommands } from "@/core/bindings/commands/exports";
import { ExportSourceContent, ExportsProject } from "@/core/bindings/types/xrf-export";
import { transformError } from "@/core/error/lib";
import { releaseEditorProject } from "@/core/ipc/release";
import { emitNotification, ENotificationSeverity } from "@/core/notifications/lib";
import { EApplicationId } from "@/core/routing/application";
import { createLoadable, Loadable } from "@/lib/loadable";
import { Logger } from "@/lib/logging";
import { call, ExclusiveFlow, TFlow } from "@/lib/mobx";
import { Nullable } from "@/lib/types/general";

@Injectable()
export class ExportsService {
  public readonly log: Logger = new Logger(__MODULE_NAME__);

  @Observable()
  public isReady: boolean = false;

  @Observable()
  public project: Loadable<Nullable<ExportsProject>> = createLoadable(null);

  public constructor(private readonly eventBus: EventBus = inject(EventBus)) {}

  @OnProvision()
  public async onProvision(): Promise<void> {
    await flowResult(this.restore());
  }

  @OnDeactivation()
  public onDeactivation(): void {
    releaseEditorProject(exportsCommands.closeProject);
  }

  /**
   * Puts back whatever the backend already had open.
   *
   * Exclusive rather than latest. A restore must lose to anything the user started: joining the lane
   * leaves an open in progress alone, where superseding would cancel the very thing the user asked for. The user's
   * own actions take the lane the other way round, so an open cancels a restore that is still in flight.
   */
  @ExclusiveFlow("project")
  private *restore(): TFlow {
    try {
      const project: Nullable<ExportsProject> = yield* call(exportsCommands.getProject());

      this.log.info(project ? "Existing exports project detected" : "No existing exports project");

      this.project = createLoadable(project);
      this.isReady = true;
    } catch (error: unknown) {
      const transformed: Error = transformError(error);

      this.log.error("Failed to restore exports project:", transformed);

      this.project = this.project.asFailed(transformed, null);
      this.isReady = true;

      emitNotification(this.eventBus, {
        details: transformed.message,
        severity: ENotificationSeverity.ERROR,
        source: EApplicationId.EXPORTS_EXPLORER,
        title: "Could not restore the open exports project",
      });
    }
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

  @ExclusiveFlow("project")
  public *openExportsProject(path: string): TFlow {
    this.log.info("Parsing exports from project:", path);
    this.project = this.project.asLoading(null);

    try {
      const result: ExportsProject = yield* call(exportsCommands.openProject(path));

      this.project = this.project.asReady(result);
    } catch (error: unknown) {
      const transformed: Error = transformError(error);

      this.log.error("Failed to parse exports:", transformed);

      this.project = this.project.asFailed(transformed, null);

      emitNotification(this.eventBus, {
        details: `${path}\n${transformed.message}`,
        severity: ENotificationSeverity.ERROR,
        source: EApplicationId.EXPORTS_EXPLORER,
        title: "Could not parse exports",
      });
    }
  }

  @ExclusiveFlow("project")
  public *refreshExportsProject(): TFlow {
    const existing: Nullable<ExportsProject> = this.project.value;

    if (!existing) {
      return;
    }

    this.log.info("Refreshing exports project:", existing.root);
    this.project = this.project.asLoading(existing);

    try {
      const result: ExportsProject = yield* call(exportsCommands.openProject(existing.root));

      this.project = this.project.asReady(result);
    } catch (error: unknown) {
      const transformed: Error = transformError(error);

      this.log.error("Failed to refresh exports project:", transformed);

      this.project = this.project.asFailed(transformed, existing);

      emitNotification(this.eventBus, {
        details: `${existing.root}\n${transformed.message}`,
        severity: ENotificationSeverity.ERROR,
        source: EApplicationId.EXPORTS_EXPLORER,
        title: "Could not refresh exports",
      });
    }
  }

  @ExclusiveFlow("project")
  public *closeExportsProject(): TFlow {
    const existing: Nullable<ExportsProject> = this.project.value;

    this.log.info("Closing exports project");
    this.project = this.project.asLoading(existing);

    try {
      yield* call(exportsCommands.closeProject());
      // Cleared on purpose: closing swaps the viewer for the application's picker in place. It used to
      // hold the project until the caller navigated away, because clearing it unmounted the editor
      // before React Router could process that navigation. Nothing navigates on close any more.
      this.project = this.project.asReady(null);
    } catch (error: unknown) {
      const transformed: Error = transformError(error);

      this.log.error("Failed to close exports project:", transformed);

      this.project = this.project.asReady(existing);

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
