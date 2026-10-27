import { Injectable, OnProvision } from "@wirestate/core";
import { BoundAction, Computed, flowResult } from "@wirestate/mobx";
import { cancelFlow, ExclusiveFlow, LatestFlow, TFlow } from "@xrf/flow";
import { archivesCommands } from "@xrf/ipc/commands/archives";
import { requireSessionId } from "@xrf/ipc/session";
import {
  ArchiveOverrideReport,
  ArchiveResolution,
  ARCHIVES_JOB_EXTRACT,
  ArchiveSubject,
  ArchiveWorldEntry,
  EArchiveSubject,
} from "@xrf/ipc/types/xrf-app-archives";
import { SessionId, SessionSnapshot } from "@xrf/ipc/types/xrf-app-core";
import { ArchiveSharedPayload } from "@xrf/ipc/types/xrf-archive";
import { ArchiveStatistics } from "@xrf/ipc/types/xrf-archive-stats";
import { ArchiveExtractDirectoryResult } from "@xrf/ipc/types/xrf-pack";
import { XrayPathCollision, XrayRoots } from "@xrf/ipc/types/xrf-vfs";
import { Logger } from "@xrf/logging";

import { describeExtractOutcome } from "@/applications/archives-explorer/lib/describe-extract-outcome";
import { IArchiveEntry, listSubjectEntries } from "@/core/archive/lib";
import { describeRoots } from "@/core/assets/lib";
import { IJobNotice, IJobOutcome } from "@/core/jobs/lib";
import { JobOperation } from "@/core/jobs/lib/job-operation";
import { CommandOperation } from "@/core/operations/lib/command-operation";
import { SessionRootResource } from "@/core/session/lib/session-root-resource";

/**
 * What the archives explorer has open, what the side panels and dialogs report of it, and what is written out of it.
 * Which file is looked at is `ArchivePreviewService`'s.
 */
@Injectable()
export class ArchivesService {
  public readonly log: Logger = new Logger(__MODULE_NAME__);

  /**
   * What the explorer is browsing: a volume set, or a mounted world. A world is listed again once a commit lists it
   * anew; a volume set follows nothing, so no change names it.
   */
  public readonly subject: SessionRootResource<ArchiveSubject> = new SessionRootResource(this.log, {
    subject: "archives",
    close: archivesCommands.closeSubject,
    restore: archivesCommands.restoreSubject,
    follow: {
      reread: archivesCommands.restoreSubject,
      staleNotice: "Could not list the changed game folder again",
      refreshed: () => this.onListedAgain(),
    },
    adopted: () => this.loadPanels(),
  });

  /** One file written out of the open subject, to the path it answers. */
  public readonly fileExtraction: CommandOperation<string> = new CommandOperation(this.log);

  /**
   * A directory, or the whole tree, written out of the open subject: as much as an unpack writes, so a job with the same
   * identity, lease and cancel control.
   */
  public readonly extraction: JobOperation<ArchiveExtractDirectoryResult> = new JobOperation(this.log, {
    kinds: [ARCHIVES_JOB_EXTRACT],
  });

  /** What the open subject answers twice over, and what it cannot reach at all. */
  public readonly overrides: CommandOperation<ArchiveOverrideReport> = new CommandOperation(this.log);

  /** Payloads several entries of the open volume set read at once; never read for a world, which has no name table. */
  public readonly sharedPayloads: CommandOperation<Array<ArchiveSharedPayload>> = new CommandOperation(this.log);

  /** What the open subject holds, broken down. Read when a surface first asks, and kept while the subject is open. */
  public readonly statistics: CommandOperation<ArchiveStatistics> = new CommandOperation(this.log);

  /** Where the open subject looks for an engine path, in the order it looks. Read and kept the same way. */
  public readonly resolution: CommandOperation<ArchiveResolution> = new CommandOperation(this.log);

  /**
   * @returns The files the open subject holds, empty when nothing is open.
   */
  @Computed()
  public get entries(): Array<IArchiveEntry> {
    return listSubjectEntries(this.subject.value);
  }

  /** Engine paths the open subject answers with more than one copy, winner first. */
  @Computed()
  public get overridden(): Array<ArchiveWorldEntry> {
    return this.overrides.value?.overridden ?? [];
  }

  /** Copies the open subject holds that no engine lookup can reach. */
  @Computed()
  public get unreachable(): Array<XrayPathCollision> {
    return this.overrides.value?.unreachable ?? [];
  }

  /**
   * @returns Whether a write runs, which another would race: a file, or a directory however its run was found.
   */
  @Computed()
  public get isWriting(): boolean {
    return this.fileExtraction.isLoading || this.extraction.isRunning;
  }

  @OnProvision()
  public async onProvision(): Promise<void> {
    await flowResult(this.subject.restore());
  }

  /**
   * Opens a set of `.db` volumes at a path: one volume, or every volume beneath a directory.
   *
   * @param path - Volume or directory of volumes to index.
   * @returns Whether they opened.
   */
  @BoundAction()
  public openVolumes(path: string): Promise<boolean> {
    this.forgetReports();

    return flowResult(this.subject.open(path, archivesCommands.openVolumes, path));
  }

  /**
   * Opens a game folder as the engine mounts it, so a loose `gamedata` tree stands in front of the archives behind it.
   *
   * @param roots - Where to read from, as the open form named it.
   * @returns Whether it opened.
   */
  @BoundAction()
  public openWorld(roots: XrayRoots): Promise<boolean> {
    this.forgetReports();

    return flowResult(this.subject.open(describeRoots(roots), archivesCommands.openWorld, roots));
  }

  @BoundAction()
  public async closeSubject(): Promise<void> {
    if (await flowResult(this.subject.close())) {
      this.forgetReports();
    }
  }

  /**
   * Extracts one file to a destination path.
   *
   * @param entry - File to extract.
   * @param destination - Output file path.
   */
  @ExclusiveFlow("fileExtraction")
  public *extractFile(entry: IArchiveEntry, destination: string): TFlow {
    // One write at a time over the open subject, whichever kind runs.
    if (this.isWriting) {
      return;
    }

    yield* this.fileExtraction.run({
      name: "Extract archive file",
      subject: destination,
      command: () =>
        archivesCommands.extractFile(this.requireSubjectSession(), entry.name, destination).then(() => destination),
      failure: `Could not extract ${entry.name}`,
      success: () => ({ details: destination, title: `Extracted ${entry.name}` }),
    });
  }

  /**
   * Extracts files beneath a directory into a destination root. An empty prefix extracts the whole tree.
   *
   * @param prefix - Engine directory prefix; an empty string selects the tree root.
   * @param destination - Output directory path.
   */
  @ExclusiveFlow("extraction")
  public *extractDirectory(prefix: string, destination: string): TFlow {
    // One write at a time over the open subject, whichever kind runs.
    if (this.isWriting) {
      return;
    }

    this.log.info("Extracting directory:", prefix || "<root>", destination);

    const request = { sessionId: this.requireSubjectSession(), prefix, destination };

    yield* this.extraction.run({
      kind: ARCHIVES_JOB_EXTRACT,
      invoke: (id: string, progress) => archivesCommands.extractDirectory(request, id, progress),
      describe: (outcome: IJobOutcome<ArchiveExtractDirectoryResult>): IJobNotice =>
        describeExtractOutcome(prefix, destination, outcome),
    });
  }

  /** Forgets how the last writes ended, for a surface that no longer shows them. */
  @BoundAction()
  public clearExtractions(): void {
    this.fileExtraction.reset();
    this.extraction.reset();
  }

  /**
   * Reads the breakdown of the open subject, once.
   */
  @LatestFlow("statistics")
  public *loadStatistics(): TFlow {
    if (!this.statistics.value && !this.statistics.isLoading) {
      yield* this.readReport(
        this.statistics,
        "Describe archives statistics",
        "Could not describe what the archives hold",
        archivesCommands.describeStatistics
      );
    }
  }

  /**
   * Reads where the open subject looks for an engine path, once.
   */
  @LatestFlow("resolution")
  public *loadResolution(): TFlow {
    if (!this.resolution.value && !this.resolution.isLoading) {
      yield* this.readReport(
        this.resolution,
        "Describe archives resolution",
        "Could not describe where the archives look for a file",
        archivesCommands.describeResolution
      );
    }
  }

  /**
   * Lets every report of the subject held go, a read still running included, so a subject opened next or none at all
   * shows none of it.
   */
  @BoundAction()
  private forgetReports(): void {
    cancelFlow(this, "overrides");
    this.overrides.reset();
    this.sharedPayloads.reset();
    this.forgetDialogReports();
  }

  /**
   * Lets the reports the dialogs read once go, a read still running included: its answer is of a listing no longer
   * shown, and the next ask reads them of the one that is.
   */
  @BoundAction()
  private forgetDialogReports(): void {
    cancelFlow(this, "statistics");
    cancelFlow(this, "resolution");
    this.statistics.reset();
    this.resolution.reset();
  }

  /** A world listed anew: what describes its listing is read again, the dialogs' reports when next asked for. */
  @BoundAction()
  private onListedAgain(): void {
    this.forgetDialogReports();

    void flowResult(this.reloadPanels());
  }

  /** Reads what the side panels report again, for a subject listed anew; a later listing supersedes it. */
  @LatestFlow("overrides")
  private *reloadPanels(): TFlow {
    yield* this.loadPanels();
  }

  /**
   * Reads what the side panels report about the open subject, inside whichever flow opened it.
   *
   * Undecorated on purpose: these belong to the open that asked for them, so a superseding open cancels them with the
   * rest of its own work instead of racing it from a lane of its own.
   */
  private *loadPanels(): TFlow {
    yield* this.readReport(
      this.overrides,
      "List archive overrides",
      "Could not list the overridden archive files",
      archivesCommands.listOverrides
    );

    // A name table only. A world derives nothing about shared payloads, so asking would answer a question it cannot
    // see rather than answer it with an empty list.
    if (this.subject.value?.kind === EArchiveSubject.VOLUMES) {
      yield* this.readReport(
        this.sharedPayloads,
        "List archive shared payloads",
        "Could not list the archives' shared payloads",
        archivesCommands.listSharedPayloads
      );
    }
  }

  /**
   * Reads one report of the open subject into its operation.
   *
   * @param report - The operation holding the report.
   * @param name - What the read is, as its log lines name it.
   * @param failure - The failure notice's title.
   * @param query - The report's query, addressed by the subject's session.
   */
  private *readReport<T>(
    report: CommandOperation<T>,
    name: string,
    failure: string,
    query: (sessionId: SessionId) => Promise<SessionSnapshot<T>>
  ): TFlow {
    yield* report.run({
      name,
      failure,
      command: () => query(this.requireSubjectSession()).then((answer: SessionSnapshot<T>) => answer.value),
    });
  }

  /** The session every read and write of the open subject is addressed by. */
  private requireSubjectSession(): SessionId {
    return requireSessionId(this.subject.snapshot);
  }
}
