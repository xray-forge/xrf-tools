import { Injectable } from "@wirestate/core";
import { ExclusiveFlow, TFlow } from "@xrf/flow";
import { renderCommands } from "@xrf/ipc/commands/render";
import { systemCommands } from "@xrf/ipc/commands/system";
import { HostInfo } from "@xrf/ipc/types/xrf-app-system";
import { BuildInfo } from "@xrf/ipc/types/xrf-build-info";
import { EnvFlagReading } from "@xrf/ipc/types/xrf-env-flags";
import { RenderLibraries } from "@xrf/ipc/types/xrf-renderer";
import { Logger } from "@xrf/logging";

import { ENotificationSeverity } from "@/core/notifications/lib";
import { CommandOperation } from "@/core/operations/lib/command-operation";
import { ICommandReport } from "@/core/operations/lib/command-reporter";

/**
 * What the About section reports of the running instance that the backend alone knows: the build, the host, the
 * environment variables it reads and the graphics libraries. None of it changes while the application runs, so each is
 * read once and kept; one that failed is read again the next time the section shows.
 */
@Injectable()
export class SettingsAboutService {
  public readonly log: Logger = new Logger(__MODULE_NAME__);

  /** Which build is running, and where it came from. */
  public readonly build: CommandOperation<BuildInfo> = new CommandOperation(this.log);

  /** The machine and the runtime the application found when it started. */
  public readonly host: CommandOperation<HostInfo> = new CommandOperation(this.log);

  /** The environment variables the application reads, as set for this run. */
  public readonly variables: CommandOperation<Array<EnvFlagReading>> = new CommandOperation(this.log);

  /** The graphics libraries the renderer loaded, the DLSS version among them. */
  public readonly libraries: CommandOperation<RenderLibraries> = new CommandOperation(this.log);

  /** Reads whatever is not known yet. */
  @ExclusiveFlow()
  public *read(): TFlow {
    yield* this.readOnce(this.build, {
      name: "Read build info",
      command: () => systemCommands.getBuildInfo(),
      failure: "Could not read the build",
    });
    yield* this.readOnce(this.host, {
      name: "Read host info",
      command: () => systemCommands.getHostInfo(),
      failure: "Could not read the environment",
    });
    yield* this.readOnce(this.variables, {
      name: "Read environment variables",
      command: () => systemCommands.getEnvironmentFlags(),
      failure: "Could not read the environment variables",
    });
    yield* this.readOnce(this.libraries, {
      name: "Read graphics libraries",
      command: () => renderCommands.describeLibraries(),
      failure: "Could not read the graphics libraries",
    });
  }

  /** Runs a read the section did not ask for by hand, unless its answer is already kept. */
  private *readOnce<T>(operation: CommandOperation<T>, report: ICommandReport<T>): TFlow {
    if (operation.value === null) {
      yield* operation.run({ ...report, failureSeverity: ENotificationSeverity.WARNING });
    }
  }
}
