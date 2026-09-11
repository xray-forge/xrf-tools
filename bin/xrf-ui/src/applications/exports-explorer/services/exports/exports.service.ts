import { EventBus, inject, Injectable, OnDeactivation, OnProvision } from "@wirestate/core";
import { BoundAction, flowResult, Observable } from "@wirestate/mobx";

import { exportsCommands } from "@/core/bindings/commands/exports";
import { ExportSourceContent, ExportsProject } from "@/core/bindings/types/xrf-export";
import { transformError } from "@/core/error/lib";
import { DocumentSession, requireDocumentSession, restoreDocument, TDocument } from "@/core/ipc/document";
import { releaseEditorProject } from "@/core/ipc/release";
import { emitNotification, ENotificationSeverity } from "@/core/notifications/lib";
import { EApplicationId } from "@/core/routing/application";
import { Loadable } from "@/lib/loadable";
import { Logger } from "@/lib/logging";
import { call, ExclusiveFlow, LatestFlow, TFlow } from "@/lib/mobx";
import { Nullable } from "@/lib/types/general";

@Injectable()
export class ExportsService {
  public readonly log: Logger = new Logger(__MODULE_NAME__);

  private readonly session: DocumentSession = new DocumentSession((ids) => exportsCommands.closeProject(ids));

  @Observable()
  public isReady: boolean = false;

  @Observable()
  public project: Loadable<Nullable<TDocument<ExportsProject>>> = Loadable.idle(null);

  public constructor(private readonly eventBus: EventBus = inject(EventBus)) {}

  @OnProvision()
  public async onProvision(): Promise<void> {
    await flowResult(this.restore());
  }

  @OnDeactivation()
  public onDeactivation(): void {
    releaseEditorProject(() => this.session.close(this.project.value?.sessionId));
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
      const project: Nullable<TDocument<ExportsProject>> = yield* call(
        exportsCommands.getProject().then((snapshot) => (snapshot ? restoreDocument(snapshot) : null))
      );

      this.log.info(project ? "Existing exports project detected" : "No existing exports project");

      this.project = this.project.asReady(project);
      this.isReady = true;
    } catch (error: unknown) {
      const transformed: Error = transformError(error);

      this.log.error("Failed to restore exports project:", transformed);

      this.project = this.project.asFailed(transformed);
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

    return exportsCommands.getSource(requireDocumentSession(this.project.value), name);
  }

  @LatestFlow("project")
  public *openExportsProject(path: string): TFlow {
    this.log.info("Parsing exports from project:", path);
    this.project = this.project.asLoading();

    try {
      const result: TDocument<ExportsProject> = yield* call(
        this.session.open((sessionId) => exportsCommands.openProject(sessionId, path).then(restoreDocument))
      );

      this.project = this.project.asReady(result);
    } catch (error: unknown) {
      const transformed: Error = transformError(error);

      this.log.error("Failed to parse exports:", transformed);

      this.project = this.project.asFailed(transformed);

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
    const existing: Nullable<TDocument<ExportsProject>> = this.project.value;

    if (!existing) {
      return;
    }

    this.log.info("Refreshing exports project:", existing.root);
    this.project = this.project.asLoading(existing);

    try {
      const result: TDocument<ExportsProject> = yield* call(
        this.session.open((sessionId) => exportsCommands.openProject(sessionId, existing.root).then(restoreDocument))
      );

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

  @LatestFlow("project")
  public *closeExportsProject(): TFlow {
    const previous: Loadable<Nullable<TDocument<ExportsProject>>> = this.project;

    this.log.info("Closing exports project");
    this.project = this.project.asLoading();

    try {
      yield* call(this.session.close(this.project.value?.sessionId));

      // Cleared on purpose: closing swaps the viewer for the application's picker in place. It used to
      // hold the project until the caller navigated away, because clearing it unmounted the editor
      // before React Router could process that navigation. Nothing navigates on close any more.
      this.project = this.project.asIdle();
    } catch (error: unknown) {
      const transformed: Error = transformError(error);

      this.log.error("Failed to close exports project:", transformed);

      this.project = previous;

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
