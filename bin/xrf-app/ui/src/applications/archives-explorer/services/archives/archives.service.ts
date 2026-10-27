import { inject, Injectable, OnProvision } from "@wirestate/core";
import { BoundAction, Computed, flowResult, Observable } from "@wirestate/mobx";
import { AsyncState } from "@xrf/async";
import { call, cancelFlow, ExclusiveFlow, LatestFlow, TFlow } from "@xrf/flow";
import { archivesCommands } from "@xrf/ipc/commands/archives";
import { isOlderSessionAnswer, requireSessionId } from "@xrf/ipc/session";
import {
  ArchiveOverrideReport,
  ArchiveResolution,
  ARCHIVES_JOB_EXTRACT,
  ArchiveSubject,
  ArchiveWorldEntry,
  EArchiveSubject,
} from "@xrf/ipc/types/xrf-app-archives";
import { SessionId, SessionSnapshot } from "@xrf/ipc/types/xrf-app-core";
import { ArchiveDescriptor, ArchiveFileDescriptor, ArchiveSharedPayload } from "@xrf/ipc/types/xrf-archive";
import { ArchiveStatistics } from "@xrf/ipc/types/xrf-archive-stats";
import { ArchiveExtractDirectoryResult } from "@xrf/ipc/types/xrf-pack";
import { EXrayAssetContainer, XrayPathCollision, XrayRoots } from "@xrf/ipc/types/xrf-vfs";
import { Logger } from "@xrf/logging";
import { Nullable, Optional } from "@xrf/types";

import { describeExtractOutcome } from "@/applications/archives-explorer/lib/describe-extract-outcome";
import {
  ArchivePreviewSupport,
  getArchivePreviewSupport,
  getArchiveVolumeOf,
  IArchiveEntry,
  listSubjectEntries,
  TArchiveContent,
  TArchiveOperation,
  TArchiveSelection,
} from "@/core/archive/lib";
import { describeRoots } from "@/core/assets/lib";
import { IJobNotice, IJobOutcome, IJobRun, IJobState } from "@/core/jobs/lib";
import { JobsService } from "@/core/jobs/services/jobs";
import { CommandReporter, TCommandCompletion } from "@/core/operations/lib/command-reporter";
import { EPathEntryKind } from "@/core/path/entry-kind";
import { SessionRootResource } from "@/core/session/lib/session-root-resource";

import { ArchiveContentReader } from "./archives.service.content";

/** What a subject answers before one has been folded, so an idle state carries a shape rather than a null. */
const EMPTY_OVERRIDES: ArchiveOverrideReport = { overridden: [], unreachable: [] };

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

  /** The extraction this service started, while it runs. */
  @Observable()
  public jobId: Nullable<string> = null;

  @Computed()
  public get job(): Nullable<IJobState> {
    return this.jobId ? this.jobsService.getJob(this.jobId) : this.jobsService.getJobOfKind(ARCHIVES_JOB_EXTRACT);
  }

  /** What the open subject answers twice over, and what it cannot reach at all. */
  @Observable()
  public overrides: AsyncState<ArchiveOverrideReport> = AsyncState.idle(EMPTY_OVERRIDES);

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

  /** Payloads several entries of the open volume set read at once. Empty for a world, which has no name table. */
  @Observable()
  public sharedPayloads: AsyncState<Array<ArchiveSharedPayload>> = AsyncState.idle([]);

  /** What the explorer points at. Exactly one kind at a time, by construction. */
  @Observable()
  public selection: TArchiveSelection = { kind: "none" };

  /** Whatever was loaded for the selection - text, or a decoded texture or sound with its description. */
  @Observable()
  public content: AsyncState<TArchiveContent> = AsyncState.idle();

  /** The last write to disk, so whichever surface started it can report the outcome. */
  @Observable()
  public operation: AsyncState<TArchiveOperation> = AsyncState.idle();

  /** What the open subject holds, broken down. Loaded when a surface first asks, and kept while the subject is open. */
  @Observable()
  public statistics: AsyncState<Nullable<ArchiveStatistics>> = AsyncState.idle(null);

  /** Where the open subject looks for an engine path, in the order it looks. Loaded and kept the same way. */
  @Observable()
  public resolution: AsyncState<Nullable<ArchiveResolution>> = AsyncState.idle(null);

  /**
   * @returns The files the open subject holds, empty when nothing is open.
   */
  @Computed()
  public get entries(): Array<IArchiveEntry> {
    return listSubjectEntries(this.subject.value);
  }

  /**
   * @returns The selected entry, or null.
   */
  @Computed()
  public get selectedEntry(): Nullable<IArchiveEntry> {
    return this.selection.kind === EPathEntryKind.FILE ? this.selection.entry : null;
  }

  /**
   * The name-table descriptor of the selected entry, which only a volume set has.
   *
   * Looked up rather than carried on the selection: a selection is the one shape both subjects share, and widening it
   * to a union would make every surface that only shows a name narrow it first.
   *
   * @returns The descriptor, or null when a world is open or nothing is selected.
   */
  @Computed()
  public get selectedDescriptor(): Nullable<ArchiveFileDescriptor> {
    const subject: Nullable<ArchiveSubject> = this.subject.value;
    const entry: Nullable<IArchiveEntry> = this.selectedEntry;

    if (!entry || subject?.kind !== EArchiveSubject.VOLUMES) {
      return null;
    }

    return subject.project.files[entry.name] ?? null;
  }

  /**
   * The mounted-world entry selected, which carries where its bytes are and what it hides.
   *
   * @returns The entry, or null when a volume set is open or nothing is selected.
   */
  @Computed()
  public get selectedWorldEntry(): Nullable<ArchiveWorldEntry> {
    const subject: Nullable<ArchiveSubject> = this.subject.value;
    const entry: Nullable<IArchiveEntry> = this.selectedEntry;

    if (!entry || subject?.kind !== EArchiveSubject.WORLD) {
      return null;
    }

    return subject.world.files.find((candidate: ArchiveWorldEntry) => candidate.name === entry.name) ?? null;
  }

  /**
   * Where the selected file's bytes come from, and which copies that decision hides.
   *
   * @returns The origin of the selection, or null when nothing is selected.
   */
  @Computed()
  public get selectedOrigin(): Nullable<ArchiveWorldEntry> {
    const entry: Nullable<IArchiveEntry> = this.selectedEntry;

    if (!entry) {
      return null;
    }

    if (this.subject.value?.kind === EArchiveSubject.WORLD) {
      return this.selectedWorldEntry;
    }

    const volume: Nullable<ArchiveDescriptor> = getArchiveVolumeOf(
      this.subject.value?.project ?? null,
      this.selectedDescriptor
    );

    if (!volume) {
      return null;
    }

    return {
      container: { kind: EXrayAssetContainer.ARCHIVE, path: volume.path },
      name: entry.name,
      shadowed: this.overridden.find((candidate: ArchiveWorldEntry) => candidate.name === entry.name)?.shadowed ?? [],
      sizeReal: entry.sizeReal,
    };
  }

  /**
   * @returns Whether an extraction would race with another command.
   */
  @Computed()
  public get isWriting(): boolean {
    return this.operation.isLoading || this.job !== null;
  }

  /** What each previewable kind has to be asked for, which is a table rather than state. */
  private readonly reader: ArchiveContentReader = new ArchiveContentReader(() => this.requireSubjectSession());

  private readonly reporter: CommandReporter = new CommandReporter(this.log);

  public constructor(private readonly jobsService: JobsService = inject(JobsService)) {}

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
    this.forgetReads();

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
    this.forgetReads();

    return flowResult(this.subject.open(describeRoots(roots), archivesCommands.openWorld, roots));
  }

  @BoundAction()
  public async closeSubject(): Promise<void> {
    if (await flowResult(this.subject.close())) {
      this.forgetReads();
    }
  }

  @LatestFlow("content")
  public *selectArchiveFile(entry: IArchiveEntry): TFlow {
    this.log.info("Select archive file:", entry.name);

    this.selection = { kind: EPathEntryKind.FILE, entry };
    this.content = this.content.asIdle();

    yield* this.loadSelectedContent(entry);
  }

  /**
   * Stops the running extraction, if there is one.
   *
   * What it has already written stays: the destination may hold the user's own files, and nothing here can tell those
   * from this run's.
   */
  @BoundAction()
  public cancelExtraction(): void {
    const job: Nullable<IJobState> = this.job;

    if (job) {
      this.jobsService.cancel(job.id);
    }
  }

  /**
   * Selects the entry a description named, by the name the open subject lists it under.
   *
   * @param name - Entry name as the open subject lists it.
   */
  @BoundAction()
  public openArchiveFileByName(name: string): void {
    const entry: Optional<IArchiveEntry> = this.entries.find((candidate: IArchiveEntry) => candidate.name === name);

    if (entry) {
      void this.selectArchiveFile(entry);
    } else {
      this.log.info("Referenced archive file is no longer listed:", name);
    }
  }

  /**
   * Selects a directory instead of a file.
   *
   * @param path - Engine directory path; an empty string selects the tree root.
   */
  @BoundAction()
  public selectArchiveDirectory(path: string): void {
    cancelFlow(this, "content");

    this.selection = { kind: EPathEntryKind.DIRECTORY, path };
    this.content = this.content.asIdle();
    this.operation = this.operation.asIdle();
  }

  @LatestFlow("content")
  public *retrySelectedFile(): TFlow {
    const entry: Nullable<IArchiveEntry> = this.selectedEntry;

    if (entry) {
      yield* this.loadSelectedContent(entry);
    }
  }

  /**
   * Extracts one file to a destination path.
   *
   * @param entry - File to extract.
   * @param destination - Output file path.
   * @returns Resolves after the extraction outcome is published.
   */
  @ExclusiveFlow("operation")
  public *extractFile(entry: IArchiveEntry, destination: string): TFlow {
    if (this.isWriting) {
      return;
    }

    this.operation = this.operation.asLoading(null);

    const completion: TCommandCompletion<unknown> = yield* this.reporter.run({
      name: "Extract archive file",
      subject: destination,
      command: () => archivesCommands.extractFile(this.requireSubjectSession(), entry.name, destination),
      failure: `Could not extract ${entry.name}`,
      success: () => ({ details: destination, title: `Extracted ${entry.name}` }),
    });

    if (completion.error) {
      this.operation = this.operation.asFailed(completion.error, null);

      throw completion.error;
    }

    this.operation = this.operation.asReady({ kind: "extract-file", destination });
  }

  /**
   * Extracts files beneath a directory into a destination root. An empty prefix extracts the whole tree.
   *
   * @param prefix - Engine directory prefix; an empty string selects the tree root.
   * @param destination - Output directory path.
   */
  @ExclusiveFlow("operation")
  public *extractArchiveDirectory(prefix: string, destination: string): TFlow {
    if (this.isWriting) {
      return;
    }

    this.log.info("Extracting directory:", prefix || "<root>", destination);

    try {
      this.operation = this.operation.asLoading(null);

      const request = { sessionId: this.requireSubjectSession(), prefix, destination };

      // Started through the jobs service rather than invoked here: an empty prefix extracts everything, so this writes
      // as much as an unpack does and wants the same identity, lease, and cancel control.
      const run: IJobRun<ArchiveExtractDirectoryResult> = this.jobsService.run<ArchiveExtractDirectoryResult>({
        kind: ARCHIVES_JOB_EXTRACT,
        invoke: (id: string, progress) => archivesCommands.extractDirectory(request, id, progress),
        describe: (outcome: IJobOutcome<ArchiveExtractDirectoryResult>): IJobNotice =>
          describeExtractOutcome(prefix, destination, outcome),
      });

      this.jobId = run.id;

      const result: ArchiveExtractDirectoryResult = yield* call(run.promise);

      this.operation = this.operation.asReady({ kind: "extract-directory", result });
    } catch (error: unknown) {
      this.log.error("Failed to extract directory:", prefix || "<root>", error);

      this.operation = this.operation.asFailed(error as Error, null);

      throw error;
    } finally {
      this.jobId = null;
    }
  }

  /**
   * Clears the last extraction outcome.
   */
  @BoundAction()
  public clearOperation(): void {
    this.operation = this.operation.asIdle();
  }

  @BoundAction()
  public clearFileSelection(): void {
    cancelFlow(this, "content");

    this.selection = { kind: "none" };
    this.content = this.content.asIdle();
    this.operation = this.operation.asIdle();
  }

  /**
   * Loads the breakdown of the open subject, once.
   */
  @LatestFlow("statistics")
  public *loadStatistics(): TFlow {
    if (this.statistics.value || this.statistics.isLoading) {
      return;
    }

    this.statistics = this.statistics.asLoading(null);

    const completion: TCommandCompletion<Optional<ArchiveStatistics>> = yield* this.readReport(
      "Describe archives statistics",
      "Could not describe what the archives hold",
      archivesCommands.describeStatistics
    );

    if (completion.error) {
      this.statistics = this.statistics.asFailed(completion.error, null);
    } else if (completion.result !== undefined) {
      this.statistics = this.statistics.asReady(completion.result);
    }
  }

  /**
   * Loads where the open subject looks for an engine path, once.
   */
  @LatestFlow("resolution")
  public *loadResolution(): TFlow {
    if (this.resolution.value || this.resolution.isLoading) {
      return;
    }

    this.resolution = this.resolution.asLoading(null);

    const completion: TCommandCompletion<Optional<ArchiveResolution>> = yield* this.readReport(
      "Describe archives resolution",
      "Could not describe where the archives look for a file",
      archivesCommands.describeResolution
    );

    if (completion.error) {
      this.resolution = this.resolution.asFailed(completion.error, null);
    } else if (completion.result !== undefined) {
      this.resolution = this.resolution.asReady(completion.result);
    }
  }

  /**
   * Lets everything read of the subject held go: the selection, its content, the panels and the reports, so a subject
   * opened next or none at all shows none of it.
   */
  @BoundAction()
  private forgetReads(): void {
    this.clearFileSelection();
    this.overrides = this.overrides.asIdle(EMPTY_OVERRIDES);
    this.sharedPayloads = this.sharedPayloads.asIdle([]);
    // Both are loaded once and kept, so a new subject has to discard them or the dialogs describe the previous one.
    this.statistics = this.statistics.asIdle(null);
    this.resolution = this.resolution.asIdle(null);
  }

  /** A world listed anew: what describes its listing is read again, the reports when next asked for. */
  @BoundAction()
  private onListedAgain(): void {
    this.statistics = this.statistics.asIdle(null);
    this.resolution = this.resolution.asIdle(null);

    void flowResult(this.reloadPanels());
  }

  /** Reads what the side panels report again, for a subject listed anew; a later listing supersedes it. */
  @LatestFlow("overrides")
  private *reloadPanels(): TFlow {
    yield* this.loadPanels();
  }

  /**
   * Loads what the side panels report about the open subject, inside whichever flow opened it.
   *
   * Undecorated on purpose: these belong to the open that asked for them, so a superseding open cancels them with the
   * rest of its own work instead of racing it from a lane of its own.
   */
  private *loadPanels(): TFlow {
    yield* this.loadOverrides();

    // A name table only. A world derives nothing about shared payloads, so asking would answer a question it cannot
    // see rather than answer it with an empty list.
    if (this.subject.value?.kind === EArchiveSubject.VOLUMES) {
      yield* this.loadSharedPayloads();
    }
  }

  private *loadOverrides(): TFlow {
    this.overrides = this.overrides.asLoading(EMPTY_OVERRIDES);

    const completion: TCommandCompletion<Optional<ArchiveOverrideReport>> = yield* this.readReport(
      "List archive overrides",
      "Could not list the overridden archive files",
      archivesCommands.listOverrides
    );

    if (completion.error) {
      this.overrides = this.overrides.asFailed(completion.error, EMPTY_OVERRIDES);
    } else if (completion.result !== undefined) {
      this.overrides = this.overrides.asReady(completion.result);
    }
  }

  private *loadSharedPayloads(): TFlow {
    this.sharedPayloads = this.sharedPayloads.asLoading([]);

    // A volume set follows no commit, so its payloads are never older than what is shown.
    const completion: TCommandCompletion<Optional<Array<ArchiveSharedPayload>>> = yield* this.readReport(
      "List archive shared payloads",
      "Could not list the archives' shared payloads",
      archivesCommands.listSharedPayloads
    );

    if (completion.error) {
      this.sharedPayloads = this.sharedPayloads.asFailed(completion.error, []);
    } else if (completion.result !== undefined) {
      this.sharedPayloads = this.sharedPayloads.asReady(completion.result);
    }
  }

  /**
   * Reads one report of the open subject through the reporter.
   *
   * @param name - What the read is, as its log lines name it.
   * @param failure - The failure notice's title.
   * @param query - The report's query, addressed by the subject's session.
   * @returns The report, or nothing for one read from an older listing than the one shown, having lost a race to a
   *   commit's listing anew: the reads that listing started answer instead.
   */
  private *readReport<T>(
    name: string,
    failure: string,
    query: (sessionId: SessionId) => Promise<SessionSnapshot<T>>
  ): TFlow<TCommandCompletion<Optional<T>>> {
    return yield* this.reporter.run({
      name,
      failure,
      command: () =>
        query(this.requireSubjectSession()).then((answer: SessionSnapshot<T>) =>
          this.isOlderThanShown(answer) ? undefined : answer.value
        ),
    });
  }

  /**
   * Loads a selected file in its supported preview representation.
   *
   * @param entry - Selected file to preview.
   */
  private *loadSelectedContent(entry: IArchiveEntry): TFlow {
    const subject: Nullable<ArchiveSubject> = this.subject.value;

    if (!subject) {
      return;
    }

    const support: ArchivePreviewSupport = getArchivePreviewSupport(entry);

    switch (support.kind) {
      case "audio":
      case "texture":
      case "image":
        return yield* this.readContent(entry, support.kind);
      case "supported":
        return yield* this.readContent(entry, "text");
      case "description":
        return yield* this.readContent(entry, "description");
      case "model":
      case "too-large":
        return;
    }
  }

  /**
   * Loads and publishes one file preview.
   *
   * @param entry - File to read.
   * @param kind - Preview representation to request from the backend.
   */
  private *readContent(entry: IArchiveEntry, kind: TArchiveContent["kind"]): TFlow {
    this.content = this.content.asLoading(null);

    const completion: TCommandCompletion<TArchiveContent> = yield* this.reporter.run({
      name: `Read archive ${kind}`,
      subject: entry.name,
      command: () => this.reader.read(entry, kind),
      failure: `Could not read ${entry.name}`,
    });

    this.content = completion.error
      ? this.content.asFailed(completion.error, null)
      : this.content.asReady(completion.result);
  }

  /** The session every read and write of the open subject is addressed by. */
  private requireSubjectSession(): SessionId {
    return requireSessionId(this.subject.snapshot);
  }

  /**
   * @param answer - A report read of the open subject.
   * @returns Whether it was read from an older listing than the one shown, having lost a race to a commit's listing
   *   anew; the reads that listing started answer instead, so the report is dropped.
   */
  private isOlderThanShown(answer: SessionSnapshot<unknown>): boolean {
    const held: Nullable<SessionSnapshot<ArchiveSubject>> = this.subject.snapshot;

    return Boolean(held && isOlderSessionAnswer(held, answer));
  }
}
