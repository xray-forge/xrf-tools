import { EventBus, inject, Injectable, OnDeactivation, OnProvision } from "@wirestate/core";
import { BoundAction, Computed, flowResult, Observable } from "@wirestate/mobx";

import { transformError } from "@/core/error/lib";
import { exportsCommands } from "@/core/ipc/commands/exports";
import { requireSessionId, Session } from "@/core/ipc/session";
import { SessionSnapshot } from "@/core/ipc/types/xrf-app";
import { ExportSourceContent, ExportsProject } from "@/core/ipc/types/xrf-export";
import { emitNotification, ENotificationSeverity } from "@/core/notifications/lib";
import { EApplicationId } from "@/core/routing/application";
import { AsyncState } from "@/lib/async-state";
import { Logger } from "@/lib/logging";
import { call, ExclusiveFlow, LatestFlow, TFlow } from "@/lib/mobx";
import { Nullable } from "@/lib/types/general";

@Injectable()
export class ExportsService {
  public readonly log: Logger = new Logger(__MODULE_NAME__);

  private readonly session: Session = new Session(exportsCommands.closeProject);

  @Observable()
  public isReady: boolean = false;

  /** The last manifest written to disk, so a surface can report the outcome and block a second write. */
  @Observable()
  public manifest: AsyncState<string> = AsyncState.idle();

  @Observable()
  private projectState: AsyncState<SessionSnapshot<ExportsProject>> = AsyncState.idle();

  @Computed()
  public get project(): AsyncState<ExportsProject> {
    return this.projectState.map((snapshot) => snapshot.value);
  }

  public constructor(private readonly eventBus: EventBus = inject(EventBus)) {}

  @OnProvision()
  public async onProvision(): Promise<void> {
    await flowResult(this.restore());
  }

  @OnDeactivation()
  public onDeactivation(): void {
    this.log.info("Deactivating, release");

    this.session.release();
  }

  /**
   * Restores the committed session without superseding a user action in the same flow.
   */
  @ExclusiveFlow("project")
  private *restore(): TFlow {
    try {
      const project: Nullable<SessionSnapshot<ExportsProject>> = yield* call(exportsCommands.getProject());

      this.log.info(project ? "Existing exports project detected" : "No existing exports project");

      this.isReady = true;
      this.session.adopt(project);
      this.projectState = this.projectState.asReady(project);
    } catch (error: unknown) {
      const transformed: Error = transformError(error);

      this.log.error("Failed to restore exports project:", transformed);

      this.projectState = this.projectState.asFailed(transformed);
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

    return exportsCommands.getSource(requireSessionId(this.projectState.value), name);
  }

  /**
   * Writes the open project's externs out as one of the manifests the CLI publishes.
   *
   * @param path - Artifact to create or replace, named with the extension of the format it should hold.
   */
  @ExclusiveFlow("manifest")
  public *exportManifest(path: string): TFlow {
    const project: Nullable<SessionSnapshot<ExportsProject>> = this.projectState.value;

    if (!project) {
      return;
    }

    this.log.info("Exporting externs manifest:", path);
    this.manifest = this.manifest.asLoading();

    try {
      yield* call(exportsCommands.exportManifest(project.sessionId, path));

      this.manifest = this.manifest.asReady(path);

      emitNotification(this.eventBus, {
        details: path,
        severity: ENotificationSeverity.SUCCESS,
        source: EApplicationId.EXPORTS_EXPLORER,
        title: `Exported ${project.value.declarations.length} externs`,
      });
    } catch (error: unknown) {
      const transformed: Error = transformError(error);

      this.log.error("Failed to export externs manifest:", transformed);

      this.manifest = this.manifest.asFailed(transformed);

      emitNotification(this.eventBus, {
        details: `${path}\n${transformed.message}`,
        severity: ENotificationSeverity.ERROR,
        source: EApplicationId.EXPORTS_EXPLORER,
        title: "Could not export externs",
      });
    }
  }

  @LatestFlow("project")
  public *openExportsProject(path: string): TFlow {
    this.log.info("Parsing exports from project:", path);
    this.projectState = this.projectState.asLoading();

    try {
      const result: SessionSnapshot<ExportsProject> = yield* call(this.session.open(exportsCommands.openProject, path));

      this.projectState = this.projectState.asReady(result);
    } catch (error: unknown) {
      const transformed: Error = transformError(error);

      this.log.error("Failed to parse exports:", transformed);

      this.projectState = this.projectState.asFailed(transformed);

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
    const existing: Nullable<SessionSnapshot<ExportsProject>> = this.projectState.value;

    if (!existing) {
      return;
    }

    this.log.info("Refreshing exports project:", existing.value.root);
    this.projectState = this.projectState.asLoading(existing);

    try {
      const result: SessionSnapshot<ExportsProject> = yield* call(
        this.session.open(exportsCommands.openProject, existing.value.root)
      );

      this.projectState = this.projectState.asReady(result);
    } catch (error: unknown) {
      const transformed: Error = transformError(error);

      this.log.error("Failed to refresh exports project:", transformed);

      this.projectState = this.projectState.asFailed(transformed, existing);

      emitNotification(this.eventBus, {
        details: `${existing.value.root}\n${transformed.message}`,
        severity: ENotificationSeverity.ERROR,
        source: EApplicationId.EXPORTS_EXPLORER,
        title: "Could not refresh exports",
      });
    }
  }

  @LatestFlow("project")
  public *closeExportsProject(): TFlow {
    const previous: AsyncState<SessionSnapshot<ExportsProject>> = this.projectState;

    this.log.info("Closing exports project");
    this.projectState = this.projectState.asLoading();

    try {
      yield* call(this.session.close());

      this.projectState = this.projectState.asIdle();
      // The artifact described the project that just closed, so the next one starts without its outcome.
      this.manifest = this.manifest.asIdle();
    } catch (error: unknown) {
      const transformed: Error = transformError(error);

      this.log.error("Failed to close exports project:", transformed);

      this.projectState = previous;

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
