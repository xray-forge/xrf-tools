import { EventBus, inject, Injectable, OnDeactivation, OnDeprovision, OnProvision, ProvisionId } from "@wirestate/core";
import { BoundAction, Computed, flowResult, Observable } from "@wirestate/mobx";
import { Nullable, Optional } from "@xrf/types";

import { describeExtractOutcome } from "@/applications/archives-explorer/lib/describe-extract-outcome";
import {
  ArchivePreviewSupport,
  getArchivePreviewSupport,
  getArchiveVolumeOf,
  getSubjectReadPolicy,
  IArchiveEntry,
  listSubjectEntries,
  TArchiveContent,
  TArchiveOperation,
  TArchiveSelection,
} from "@/core/archive/lib";
import { transformError } from "@/core/error/lib";
import { archivesCommands } from "@/core/ipc/commands/archives";
import { requireSessionId, Session } from "@/core/ipc/session";
import {
  ArchiveOverrideReport,
  ArchiveResolution,
  ArchiveSubject,
  ArchiveWorldEntry,
  EArchiveSubject,
  EJobKind,
  SessionId,
  SessionSnapshot,
} from "@/core/ipc/types/xrf-app";
import {
  ArchiveDescriptor,
  ArchiveFileDescriptor,
  ArchiveReadPolicy,
  ArchiveSharedPayload,
} from "@/core/ipc/types/xrf-archive";
import { ArchiveStatistics } from "@/core/ipc/types/xrf-archive-stats";
import { ArchiveExtractDirectoryResult } from "@/core/ipc/types/xrf-pack";
import { EXrayAssetContainer, XrayPathCollision, XrayRoots } from "@/core/ipc/types/xrf-vfs";
import { IJobNotice, IJobOutcome, IJobRun, IJobState } from "@/core/jobs/lib";
import { JobsService } from "@/core/jobs/services/jobs";
import { emitNotification, ENotificationSeverity } from "@/core/notifications/lib";
import { EPathEntryKind } from "@/core/path/entry-kind";
import { EApplicationId } from "@/core/routing/application";
import { AsyncState } from "@/lib/async-state";
import { formatDuration } from "@/lib/format/duration";
import { Logger, Timer } from "@/lib/logging";
import { call, cancelFlow, ExclusiveFlow, LatestFlow, TFlow } from "@/lib/mobx";

import { ArchiveContentReader } from "./archives.service.content";

/** What a subject answers before one has been folded, so an idle state carries a shape rather than a null. */
const EMPTY_OVERRIDES: ArchiveOverrideReport = { overridden: [], unreachable: [] };

@Injectable()
export class ArchivesService {
  public readonly log: Logger = new Logger(__MODULE_NAME__);

  private readonly session: Session = new Session(archivesCommands.closeSubject);

  /** The extraction this service started, while it runs. */
  @Observable()
  public jobId: Nullable<string> = null;

  @Computed()
  public get job(): Nullable<IJobState> {
    return this.jobId ? this.jobsService.getJob(this.jobId) : this.jobsService.getJobOfKind(EJobKind.ARCHIVES_EXTRACT);
  }

  /** Idle until restoration settles; a closed explorer is ready with no value. */
  @Observable()
  private subjectState: AsyncState<SessionSnapshot<ArchiveSubject>> = AsyncState.idle();

  /** What the explorer is browsing: a volume set, or a mounted world. */
  @Computed()
  public get subject(): AsyncState<ArchiveSubject> {
    return this.subjectState.map((snapshot) => snapshot.value);
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

  public constructor(
    private readonly eventBus: EventBus = inject(EventBus),
    private readonly jobsService: JobsService = inject(JobsService)
  ) {}

  @OnProvision()
  public async onProvision(provisionId: ProvisionId): Promise<void> {
    this.log.info("Provisioning:", provisionId);

    await flowResult(this.restore());
  }

  @OnDeprovision()
  public onDeprovision(provisionId: ProvisionId): void {
    this.log.info("Deprovisioning:", provisionId);
  }

  /** Releases this editor's subject; visual previews own their model sessions separately. */
  @OnDeactivation()
  public onDeactivation(): void {
    this.log.info("Deactivating, release");

    this.session.release();
  }

  /**
   * Restores the committed session without superseding a user action in the same flow.
   */
  @ExclusiveFlow("subject")
  private *restore(): TFlow {
    this.log.info("Restoring");

    try {
      const existing: Nullable<SessionSnapshot<ArchiveSubject>> = yield* call(archivesCommands.getSubject());

      this.log.info(existing ? `Existing archives ${existing.value.kind} detected` : "No existing archives subject");

      this.session.adopt(existing);
      this.subjectState = this.subjectState.asReady(existing);

      if (existing) {
        yield* this.loadPanels();
      }
    } catch (error: unknown) {
      this.log.error("Failed to restore archives subject:", error);

      this.subjectState = this.subjectState.asFailed(transformError(error));
    }
  }

  /**
   * Opens a set of `.db` volumes at a path: one volume, or every volume beneath a directory.
   *
   * @param path - Volume or directory of volumes to index.
   */
  @LatestFlow("subject")
  public *openVolumes(path: string): TFlow {
    yield* this.open("archive volumes", path, "Could not open archive volumes", () =>
      this.session.open(archivesCommands.openVolumes, path)
    );
  }

  /**
   * Opens a game folder as the engine mounts it, so a loose `gamedata` tree stands in front of the archives behind it.
   *
   * @param roots - Where to read from, as the open form named it.
   */
  @LatestFlow("subject")
  public *openWorld(roots: XrayRoots): TFlow {
    yield* this.open("archive world", roots.roots[0]?.path ?? "", "Could not open game folder", () =>
      this.session.open(archivesCommands.openWorld, roots)
    );
  }

  @LatestFlow("subject")
  public *closeSubject(): TFlow {
    const timer: Timer = new Timer();

    this.log.info("Closing existing archives subject");

    try {
      yield* call(this.session.close());

      this.log.info("Archives subject closed in:", formatDuration(timer.elapsed()));

      this.resetSubject();
    } catch (error: unknown) {
      this.log.error("Failed to close archives subject after:", formatDuration(timer.elapsed()), error);

      throw transformError(error);
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

  @BoundAction()
  public resetSubject(): void {
    this.log.info("Reset archives subject");

    this.clearFileSelection();
    this.subjectState = this.subjectState.asReady(null);
    this.overrides = this.overrides.asIdle(EMPTY_OVERRIDES);
    this.sharedPayloads = this.sharedPayloads.asIdle([]);
    this.statistics = this.statistics.asIdle(null);
    this.resolution = this.resolution.asIdle(null);
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

    const timer: Timer = new Timer();

    this.log.info("Extracting file:", entry.name, destination);

    try {
      this.operation = this.operation.asLoading(null);

      yield* call(archivesCommands.extractFile(this.requireSubjectSession(), entry.name, destination));

      this.log.info("File extracted in:", formatDuration(timer.elapsed()));

      this.operation = this.operation.asReady({ kind: "extract-file", destination });

      emitNotification(this.eventBus, {
        details: destination,
        severity: ENotificationSeverity.SUCCESS,
        source: EApplicationId.ARCHIVES_EXPLORER,
        title: `Extracted ${entry.name}`,
      });
    } catch (error: unknown) {
      this.log.error("Failed to extract file after:", formatDuration(timer.elapsed()), error);

      this.operation = this.operation.asFailed(transformError(error), null);

      emitNotification(this.eventBus, {
        details: `${destination}\n${transformError(error).message}`,
        severity: ENotificationSeverity.ERROR,
        source: EApplicationId.ARCHIVES_EXPLORER,
        title: `Could not extract ${entry.name}`,
      });

      throw transformError(error);
    }
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

    const timer: Timer = new Timer();

    this.log.info("Extracting directory:", prefix || "<root>", destination);

    try {
      this.operation = this.operation.asLoading(null);

      const request = { sessionId: this.requireSubjectSession(), prefix, destination };

      // Started through the jobs service rather than invoked here: an empty prefix extracts everything, so this writes
      // as much as an unpack does and wants the same identity, lease, and cancel control.
      const run: IJobRun<ArchiveExtractDirectoryResult> = this.jobsService.run<ArchiveExtractDirectoryResult>({
        kind: EJobKind.ARCHIVES_EXTRACT,
        invoke: (id: string, progress) => archivesCommands.extractDirectory(request, id, progress),
        describe: (outcome: IJobOutcome<ArchiveExtractDirectoryResult>): IJobNotice =>
          describeExtractOutcome(prefix, destination, outcome),
      });

      this.jobId = run.id;

      const result: ArchiveExtractDirectoryResult = yield* call(run.promise);

      this.log.info("Directory extracted in:", formatDuration(timer.elapsed()));

      this.operation = this.operation.asReady({ kind: "extract-directory", result });
    } catch (error: unknown) {
      this.log.error("Failed to extract directory after:", formatDuration(timer.elapsed()), error);

      this.operation = this.operation.asFailed(transformError(error), null);

      throw transformError(error);
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
   * Opens whichever subject the form asked for, which differs only in the command it dispatches.
   *
   * @param what - Subject being opened, for the log line.
   * @param path - Path the person named, for the failure notice.
   * @param failureTitle - What to call a failure in the notification.
   * @param dispatch - Sends the open, called once the previous subject has left the screen.
   */
  private *open(
    what: string,
    path: string,
    failureTitle: string,
    dispatch: () => Promise<SessionSnapshot<ArchiveSubject>>
  ): TFlow {
    const timer: Timer = new Timer();

    this.log.info(`Opening ${what}:`, path);

    try {
      this.clearFileSelection();

      this.subjectState = this.subjectState.asLoading();
      this.overrides = this.overrides.asIdle(EMPTY_OVERRIDES);
      this.sharedPayloads = this.sharedPayloads.asIdle([]);
      // Both are loaded once and kept, so a new subject has to discard them or the dialogs describe the previous one.
      this.statistics = this.statistics.asIdle(null);
      this.resolution = this.resolution.asIdle(null);

      this.subjectState = this.subjectState.asReady(yield* call(dispatch()));

      this.log.info(`Opened ${what} in:`, formatDuration(timer.elapsed()));

      yield* this.loadPanels();
    } catch (error: unknown) {
      this.log.error(`Failed to open ${what} after:`, formatDuration(timer.elapsed()), error);

      this.subjectState = this.subjectState.asFailed(transformError(error));

      emitNotification(this.eventBus, {
        details: `${path}\n${transformError(error).message}`,
        severity: ENotificationSeverity.ERROR,
        source: EApplicationId.ARCHIVES_EXPLORER,
        title: failureTitle,
      });
    }
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

  /**
   * Loads the breakdown of the open subject, once.
   */
  @LatestFlow("statistics")
  public *loadStatistics(): TFlow {
    if (this.statistics.value || this.statistics.isLoading) {
      return;
    }

    try {
      this.statistics = this.statistics.asLoading(null);

      const statistics: ArchiveStatistics = yield* call(
        archivesCommands.describeStatistics(this.requireSubjectSession())
      );

      this.log.info("Archives statistics:", statistics.overview.total.files, "files");

      this.statistics = this.statistics.asReady(statistics);
    } catch (error: unknown) {
      this.log.error("Failed to describe archives statistics:", error);

      this.statistics = this.statistics.asFailed(transformError(error), null);
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

    try {
      this.resolution = this.resolution.asLoading(null);

      const resolution: ArchiveResolution = yield* call(
        archivesCommands.describeResolution(this.requireSubjectSession())
      );

      this.log.info("Archives resolution:", resolution.sources.length, "sources");

      this.resolution = this.resolution.asReady(resolution);
    } catch (error: unknown) {
      this.log.error("Failed to describe archives resolution:", error);

      this.resolution = this.resolution.asFailed(transformError(error), null);
    }
  }

  private *loadOverrides(): TFlow {
    try {
      this.overrides = this.overrides.asLoading(EMPTY_OVERRIDES);

      const report: ArchiveOverrideReport = yield* call(archivesCommands.listOverrides(this.requireSubjectSession()));

      this.log.info(
        "Archives overrides:",
        report.overridden.length,
        "overridden,",
        report.unreachable.length,
        "unreachable"
      );

      this.overrides = this.overrides.asReady(report);
    } catch (error: unknown) {
      this.log.error("Failed to describe archives overrides:", error);

      this.overrides = this.overrides.asFailed(transformError(error), EMPTY_OVERRIDES);
    }
  }

  private *loadSharedPayloads(): TFlow {
    try {
      this.sharedPayloads = this.sharedPayloads.asLoading([]);

      const payloads: Array<ArchiveSharedPayload> = yield* call(
        archivesCommands.listSharedPayloads(this.requireSubjectSession())
      );

      this.log.info("Archives shared payloads:", payloads.length);

      this.sharedPayloads = this.sharedPayloads.asReady(payloads);
    } catch (error: unknown) {
      this.log.error("Failed to list archives shared payloads:", error);

      this.sharedPayloads = this.sharedPayloads.asFailed(transformError(error), []);
    }
  }

  /**
   * Loads a selected file in its supported preview representation.
   *
   * @param entry - Selected file to preview.
   */
  private *loadSelectedContent(entry: IArchiveEntry): TFlow {
    const subject: Nullable<ArchiveSubject> = this.subject.value;
    const policy: Nullable<ArchiveReadPolicy> = getSubjectReadPolicy(subject);

    if (!subject || !policy) {
      return;
    }

    const support: ArchivePreviewSupport = getArchivePreviewSupport(entry, policy);

    switch (support.kind) {
      case "audio":
      case "texture":
      case "image":
        return yield* this.readContent(entry, support.kind, subject);
      case "supported":
        return yield* this.readContent(entry, "text", subject);
      case "description":
        return yield* this.readContent(entry, "description", subject);
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
   * @param subject - Open subject the file is read out of.
   */
  private *readContent(entry: IArchiveEntry, kind: TArchiveContent["kind"], subject: ArchiveSubject): TFlow {
    const timer: Timer = new Timer();

    this.log.info("Reading archive content:", kind, entry.name);
    this.content = this.content.asLoading(null);

    try {
      const content: TArchiveContent = yield* call(this.reader.read(entry, kind, subject));

      this.log.info("Archive content read in:", formatDuration(timer.elapsed()));

      this.content = this.content.asReady(content);
    } catch (error: unknown) {
      this.log.error("Failed to read archive content after:", formatDuration(timer.elapsed()), entry.name, error);

      this.content = this.content.asFailed(transformError(error), null);
    }
  }

  /** The session every read and write of the open subject is addressed by. */
  private requireSubjectSession(): SessionId {
    return requireSessionId(this.subjectState.value);
  }
}
