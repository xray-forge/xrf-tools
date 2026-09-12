import { EventBus, inject, Injectable, OnDeactivation, OnProvision } from "@wirestate/core";
import { BoundAction, Computed, flowResult, Observable } from "@wirestate/mobx";

import { exportsCommands } from "@/core/bindings/commands/exports";
import { SessionSnapshot } from "@/core/bindings/types/xrf-app";
import { ExportSourceContent, ExportsProject } from "@/core/bindings/types/xrf-export";
import { transformError } from "@/core/error/lib";
import { requireSessionId, Session } from "@/core/ipc/session";
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
