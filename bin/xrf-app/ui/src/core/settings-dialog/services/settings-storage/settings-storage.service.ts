import { Injectable } from "@wirestate/core";
import { ExclusiveFlow, LatestFlow, TFlow } from "@xrf/flow";
import { systemCommands } from "@xrf/ipc/commands/system";
import { AppCacheTidy } from "@xrf/ipc/types/xrf-app-core";
import { AppStorageUsage, EAppStorageKind } from "@xrf/ipc/types/xrf-app-system";
import { Logger } from "@xrf/logging";
import { Nullable } from "@xrf/types";

import { ENotificationSeverity } from "@/core/notifications/lib";
import { CommandOperation } from "@/core/operations/lib/command-operation";
import { CommandReporter } from "@/core/operations/lib/command-reporter";
import { tidyAppCache } from "@/core/settings/lib/app-cache";

/**
 * What the application keeps in its own folder on disk: measuring it by kind, clearing what is rebuilt on its own,
 * tidying the caches by the stored limits, and showing the folder.
 */
@Injectable()
export class SettingsStorageService {
  public readonly log: Logger = new Logger(__MODULE_NAME__);

  /** The folder by kind, as last measured; the previous measure shown while the next runs. */
  public readonly usage: CommandOperation<AppStorageUsage> = new CommandOperation(this.log);

  /** What the last tidy asked here did, until the section shows again. */
  public readonly tidy: CommandOperation<AppCacheTidy> = new CommandOperation(this.log);

  /** Reports the clears and the folder shown. */
  private readonly reporter: CommandReporter = new CommandReporter(this.log);

  /** Measures the folder for a section just shown, forgetting the tidy said under the last one. */
  @LatestFlow("usage")
  public *show(): TFlow {
    this.tidy.reset();

    yield* this.measure();
  }

  /**
   * Clears one kind, then measures the folder again either way. It runs in its own lane, so reopening the section
   * never cancels it; a clear asked while one runs is ignored.
   */
  @ExclusiveFlow()
  public *clear(kind: EAppStorageKind): TFlow {
    yield* this.reporter.run({
      name: "Clear application storage",
      subject: kind,
      command: () => systemCommands.clearAppStorage(kind),
      failure: "Could not clear the application's storage",
    });

    yield* this.measure();
  }

  /** Tidies the caches now by the stored limits, then measures the folder again. */
  @ExclusiveFlow()
  public *tidyNow(): TFlow {
    yield* this.tidy.run({
      name: "Tidy application caches",
      command: () => tidyAppCache(true),
      failure: "Could not tidy the application's caches",
    });

    yield* this.measure();
  }

  /** Shows the application's folder in the file manager, once it is measured. */
  @ExclusiveFlow()
  public *reveal(): TFlow {
    const root: Nullable<string> = this.usage.value?.root ?? null;

    if (!root) {
      return;
    }

    yield* this.reporter.run({
      name: "Show application folder",
      subject: root,
      command: () => systemCommands.revealPath(root),
      failure: "Could not show the application's folder",
    });
  }

  private *measure(): TFlow {
    yield* this.usage.run({
      name: "Measure application storage",
      command: () => systemCommands.getAppStorage(),
      failure: "Could not measure the application's storage",
      failureSeverity: ENotificationSeverity.WARNING,
    });
  }
}
