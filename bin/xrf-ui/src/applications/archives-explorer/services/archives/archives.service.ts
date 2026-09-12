import { EventBus, inject, Injectable, OnDeactivation, OnDeprovision, OnProvision, ProvisionId } from "@wirestate/core";
import { BoundAction, Computed, flowResult, Observable } from "@wirestate/mobx";

import { describeExtractOutcome } from "@/applications/archives-explorer/lib/describe-extract-outcome";
import {
  EArchiveSubject,
  getArchivePreviewSupport,
  getSubjectReadPolicy,
  getSubjectRoots,
  IArchiveEntry,
  isArchiveAudio,
  isArchiveImage,
  listSubjectEntries,
  TArchiveContent,
  TArchiveOperation,
  TArchiveSelection,
} from "@/core/archive";
import { archivesCommands } from "@/core/bindings/commands/archives";
import { archivesRawCommands } from "@/core/bindings/commands/archives-raw";
import { assetsRawCommands } from "@/core/bindings/commands/assets-raw";
import { ArchiveSubject, ArchiveWorldEntry, EJobKind, SessionId, SessionSnapshot } from "@/core/bindings/types/xrf-app";
import { ArchiveFileDescriptor, ArchiveReadPolicy, ArchiveSharedPayload } from "@/core/bindings/types/xrf-archive";
import { ArchiveExtractDirectoryResult } from "@/core/bindings/types/xrf-pack";
import { XrayPathCollision, XrayRoots } from "@/core/bindings/types/xrf-vfs";
import { transformError } from "@/core/error/lib";
import { requireSessionId, Session } from "@/core/ipc/session";
import { IJobNotice, IJobOutcome, IJobRun, IJobState } from "@/core/jobs/lib";
import { JobsService } from "@/core/jobs/services/jobs";
import { emitNotification, ENotificationSeverity } from "@/core/notifications/lib";
import { EPathEntryKind } from "@/core/path/entry-kind";
import { EApplicationId } from "@/core/routing/application";
import { AsyncState } from "@/lib/async-state";
import { formatDuration } from "@/lib/format/duration";
import { Logger, Timer } from "@/lib/logging";
import { call, cancelFlow, ExclusiveFlow, LatestFlow, TFlow } from "@/lib/mobx";
import { Nullable } from "@/lib/types/general";

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

  /** Entries the open subject holds that no engine lookup can reach. */
  @Observable()
  public collisions: AsyncState<Array<XrayPathCollision>> = AsyncState.idle([]);

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
   * @returns The engine directory path, with an empty string for the tree root, or null.
   */
  @Computed()
  public get selectedDirectory(): Nullable<string> {
    return this.selection.kind === EPathEntryKind.DIRECTORY ? this.selection.path : null;
  }

  /**
   * @returns Whether an extraction would race with another command.
   */
  @Computed()
  public get isWriting(): boolean {
    return this.operation.isLoading || this.job !== null;
  }

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
    this.collisions = this.collisions.asIdle([]);
    this.sharedPayloads = this.sharedPayloads.asIdle([]);
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
      this.collisions = this.collisions.asIdle([]);
      this.sharedPayloads = this.sharedPayloads.asIdle([]);

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
    yield* this.loadCollisions();

    // A name table only. A world derives nothing about shared payloads, so asking would answer a question it cannot
    // see rather than answer it with an empty list.
    if (this.subject.value?.kind === EArchiveSubject.VOLUMES) {
      yield* this.loadSharedPayloads();
    }
  }

  private *loadCollisions(): TFlow {
    try {
      this.collisions = this.collisions.asLoading([]);

      const collisions: Array<XrayPathCollision> = yield* call(
        archivesCommands.listCollisions(this.requireSubjectSession())
      );

      this.log.info("Archives unreachable entries:", collisions.length);

      this.collisions = this.collisions.asReady(collisions);
    } catch (error: unknown) {
      this.log.error("Failed to list archives collisions:", error);

      this.collisions = this.collisions.asFailed(transformError(error), []);
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

    // todo: Switch case based on type?
    if (isArchiveAudio(entry, policy)) {
      return yield* this.readContent(entry, "audio", subject);
    } else if (isArchiveImage(entry, policy)) {
      return yield* this.readContent(entry, "image", subject);
    } else if (getArchivePreviewSupport(entry, policy).kind === "supported") {
      return yield* this.readContent(entry, "text", subject);
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
      const content: TArchiveContent = yield* call(
        kind === "audio"
          ? this.readAudioContent(entry, subject)
          : kind === "image"
            ? this.readImageContent(entry, subject)
            : this.readTextContent(entry)
      );

      this.log.info("Archive content read in:", formatDuration(timer.elapsed()));

      this.content = this.content.asReady(content);
    } catch (error: unknown) {
      this.log.error("Failed to read archive content after:", formatDuration(timer.elapsed()), entry.name, error);

      this.content = this.content.asFailed(transformError(error), null);
    }
  }

  /**
   * Reads a file as the Windows-1251 text an engine surface shows.
   *
   * @param entry - Entry naming the file.
   * @returns The decoded text.
   */
  private async readTextContent(entry: IArchiveEntry): Promise<TArchiveContent> {
    return { kind: "text", result: await archivesCommands.readFile(this.requireSubjectSession(), entry.name) };
  }

  /**
   * Reads a sound as the description the engine would read plus the bytes the webview plays.
   *
   * @param entry - Entry naming the sound.
   * @param subject - Open subject whose tree the sound is read out of.
   * @returns The sound's description and its bytes as stored.
   */
  private async readAudioContent(entry: IArchiveEntry, subject: ArchiveSubject): Promise<TArchiveContent> {
    const roots: XrayRoots = getSubjectRoots(subject);

    const [audio, bytes] = await Promise.all([
      archivesCommands.describeAudio(roots, entry.name),
      assetsRawCommands.readAsset(roots, entry.name),
    ]);

    return { kind: "audio", descriptor: audio, bytes: new Uint8Array(bytes) };
  }

  /**
   * Reads a texture as its source shape plus the png the backend decoded it into.
   *
   * @param entry - Entry naming the texture.
   * @param subject - Open subject whose tree the texture is read out of.
   * @returns The texture's shape and the decoded png bytes.
   */
  private async readImageContent(entry: IArchiveEntry, subject: ArchiveSubject): Promise<TArchiveContent> {
    const roots: XrayRoots = getSubjectRoots(subject);

    const [texture, bytes] = await Promise.all([
      archivesCommands.describeImage(roots, entry.name),
      archivesRawCommands.readImage(roots, entry.name),
    ]);

    return { kind: "image", descriptor: texture, bytes: new Uint8Array(bytes) };
  }

  /** The session every read and write of the open subject is addressed by. */
  private requireSubjectSession(): SessionId {
    return requireSessionId(this.subjectState.value);
  }
}
