import { Injectable, OnProvision } from "@wirestate/core";
import { BoundAction, flowResult } from "@wirestate/mobx";
import { ExclusiveFlow, TFlow } from "@xrf/flow";
import { formatDuration } from "@xrf/format";
import { exportsCommands } from "@xrf/ipc/commands/exports";
import { requireSessionId } from "@xrf/ipc/session";
import { SessionSnapshot } from "@xrf/ipc/types/xrf-app-core";
import { ExportSourceContent, ExportsProject } from "@xrf/ipc/types/xrf-export";
import { Logger, Timer } from "@xrf/logging";
import { Nullable } from "@xrf/types";

import { CommandOperation } from "@/core/operations/lib/command-operation";
import { SessionRootResource } from "@/core/session/lib/session-root-resource";

@Injectable()
export class ExportsService {
  public readonly log: Logger = new Logger(__MODULE_NAME__);

  public readonly project: SessionRootResource<ExportsProject> = new SessionRootResource(this.log, {
    subject: "exports project",
    close: exportsCommands.closeProject,
    restore: exportsCommands.restoreProject,
  });

  /** The last manifest written to disk, so a surface can report the outcome and block a second write. */
  public readonly manifest: CommandOperation<string> = new CommandOperation(this.log);

  @OnProvision()
  public async onProvision(): Promise<void> {
    await flowResult(this.project.restore());
  }

  /**
   * Reads the source that declares an extern.
   *
   * @param name - Declaration name reported by the project.
   * @returns Resolves to the source content that declares the extern.
   */
  @BoundAction()
  public async readExportSource(name: string): Promise<ExportSourceContent> {
    const timer: Timer = new Timer();

    try {
      const source: ExportSourceContent = await exportsCommands.getSource(
        requireSessionId(this.project.snapshot),
        name
      );

      this.log.info("Export source read:", name, "in", formatDuration(timer.elapsed()));

      return source;
    } catch (error: unknown) {
      this.log.error("Failed to read export source:", name, "after", formatDuration(timer.elapsed()), error);

      throw error;
    }
  }

  /**
   * Writes the open project's externs out as one of the manifests the CLI publishes.
   *
   * @param path - Artifact to create or replace, named with the extension of the format it should hold.
   */
  @ExclusiveFlow("manifest")
  public *exportManifest(path: string): TFlow {
    const project: Nullable<SessionSnapshot<ExportsProject>> = this.project.snapshot;

    if (!project) {
      return;
    }

    yield* this.manifest.run({
      name: "Export externs manifest",
      subject: path,
      command: () => exportsCommands.exportManifest(project.sessionId, path).then(() => path),
      failure: "Could not export externs",
      success: () => ({ title: `Exported ${project.value.declarations.length} externs`, details: path }),
    });
  }

  @BoundAction()
  public openExportsProject(path: string): Promise<boolean> {
    return flowResult(this.project.open(path, exportsCommands.openProject, path));
  }

  @BoundAction()
  public refreshExportsProject(): Promise<boolean> {
    const root: Nullable<string> = this.project.value?.root ?? null;

    return root ? flowResult(this.project.refresh(root, exportsCommands.openProject, root)) : Promise.resolve(false);
  }

  @BoundAction()
  public async closeExportsProject(): Promise<void> {
    // The manifest described the project that just closed, so the next one starts without its outcome.
    if (await flowResult(this.project.close())) {
      this.manifest.reset();
    }
  }
}
