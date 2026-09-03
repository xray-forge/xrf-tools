import { EventBus, inject, Injectable, OnDeactivation, OnDeprovision, OnProvision, ProvisionId } from "@wirestate/core";
import { BoundAction, Computed, flowResult, Observable } from "@wirestate/mobx";

import { describeExtractOutcome } from "@/applications/archives-explorer/lib/describe-extract-outcome";
import {
  createArchiveRoots,
  getArchivePreviewSupport,
  isArchiveAudio,
  isArchiveImage,
  listArchiveFiles,
  TArchiveContent,
  TArchiveOperation,
  TArchiveSelection,
} from "@/core/archive";
import { archivesCommands } from "@/core/bindings/commands/archives";
import { archivesRawCommands } from "@/core/bindings/commands/archives-raw";
import { assetsRawCommands } from "@/core/bindings/commands/assets-raw";
import { visualsCommands } from "@/core/bindings/commands/visuals";
import { ArchiveFileDescriptor, ArchiveProject, ArchiveSharedPayload } from "@/core/bindings/types/xrf-archive";
import { ArchiveExtractDirectoryResult } from "@/core/bindings/types/xrf-pack";
import { XrayPathCollision, XrayRoots } from "@/core/bindings/types/xrf-vfs";
import { transformError } from "@/core/error/lib";
import { releaseEditorProject } from "@/core/ipc/release";
import { EJobKind, IJobNotice, IJobOutcome, IJobRun, IJobState } from "@/core/jobs/lib";
import { JobsService } from "@/core/jobs/services/jobs";
import { emitNotification, ENotificationSeverity } from "@/core/notifications/lib";
import { EApplicationId } from "@/core/routing/application";
import { formatDuration } from "@/lib/format/duration";
import { createLoadable, Loadable } from "@/lib/loadable";
import { Logger, Timer } from "@/lib/logging";
import { call, cancelFlow, ExclusiveFlow, LatestFlow, TFlow } from "@/lib/mobx";
import { Nullable } from "@/lib/types/general";

@Injectable()
export class ArchivesService {
  public readonly log: Logger = new Logger(__MODULE_NAME__);

  @Observable()
  public isReady: boolean = false;

  @Observable()
  public project: Loadable<Nullable<ArchiveProject>> = createLoadable(null);

  /**
   * Entries the open volume set holds that no engine lookup can reach.
   *
   * Loaded beside the project rather than with it: the project is a name table keyed as authored, and what those names
   * fold to is the backend's mount layer to answer. A failure here is reported and dropped - a volume set that opened
   * is still browsable when nobody could tell what is unreachable in it.
   */
  @Observable()
  public collisions: Loadable<Array<XrayPathCollision>> = createLoadable([]);

  /**
   * Payloads several entries of the open volume set read at once.
   */
  @Observable()
  public sharedPayloads: Loadable<Array<ArchiveSharedPayload>> = createLoadable([]);

  /** What the explorer points at. Exactly one kind at a time, by construction. */
  @Observable()
  public selection: TArchiveSelection = { kind: "none" };

  /** Whatever was loaded for the selection - text, or a decoded texture or sound with its description. */
  @Observable()
  public content: Loadable<Nullable<TArchiveContent>> = createLoadable(null);

  /** The last write to disk, so whichever surface started it can report the outcome. */
  @Observable()
  public operation: Loadable<Nullable<TArchiveOperation>> = createLoadable(null);

  /**
   * Returns the files the opened project holds, without the directories its volumes record.
   *
   * @returns Descriptors of the entries that are files, empty when no project is open.
   */
  @Computed()
  public get files(): Array<ArchiveFileDescriptor> {
    return listArchiveFiles(this.project.value);
  }

  /**
   * Returns the selected file, or null when a directory or nothing is selected.
   *
   * @returns The selected file descriptor, or null.
   */
  @Computed()
  public get selectedFile(): Nullable<ArchiveFileDescriptor> {
    return this.selection.kind === "file" ? this.selection.descriptor : null;
  }

  /**
   * Returns the selected directory, or null when a file or nothing is selected.
   *
   * @returns The archive-relative directory path, with an empty string for the archive root, or null.
   */
  @Computed()
  public get selectedDirectory(): Nullable<string> {
    return this.selection.kind === "directory" ? this.selection.path : null;
  }

  /**
   * Reports whether a write to disk is in progress.
   *
   * Only a write holds the next open back. A read in flight does not: the `content` lane supersedes it, so the newer
   * gesture cancels the older read rather than being dropped for it.
   *
   * @returns Whether an extraction would race with another command.
   */
  @Computed()
  public get isWriting(): boolean {
    return this.operation.isLoading;
  }

  /** The extraction this service started, while it runs. */
  @Observable()
  public jobId: Nullable<string> = null;

  public constructor(
    private readonly eventBus: EventBus = inject(EventBus),
    private readonly jobsService: JobsService = inject(JobsService)
  ) {}

  /**
   * @returns The extraction currently running, whether this service started it or found it again.
   */
  @Computed()
  public get job(): Nullable<IJobState> {
    return this.jobId ? this.jobsService.getJob(this.jobId) : this.jobsService.getJobOfKind(EJobKind.ARCHIVES_EXTRACT);
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

  @OnProvision()
  public async onProvision(provisionId: ProvisionId): Promise<void> {
    this.log.info("Provisioning:", provisionId);

    await flowResult(this.restore());
  }

  @OnDeprovision()
  public onDeprovision(provisionId: ProvisionId): void {
    this.log.info("Deprovisioning:", provisionId);
  }

  /**
   * Releases the archive project when the editor deactivates, and the model its preview opened.
   *
   * Previewing an archived `.ogf` goes through the shared `visuals_open_model`, which parks the model in the backend's
   * one visual selection.
   */
  @OnDeactivation()
  public onDeactivation(): void {
    this.log.info("Deactivating, release archive project");

    releaseEditorProject(archivesCommands.closeProject);
    releaseEditorProject(visualsCommands.closeModel);
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
    const existing: Nullable<ArchiveProject> = yield* call(archivesCommands.getProject());

    this.log.info(existing ? "Existing archives project detected" : "No existing archives project");

    if (existing) {
      this.project = createLoadable(existing);

      yield* this.loadCollisions();
      yield* this.loadSharedPayloads();
    }

    this.isReady = true;
  }

  @BoundAction()
  public resetArchivesProject(): void {
    this.log.info("Reset archives project");

    this.clearFileSelection();
    this.project = createLoadable(null);
    this.collisions = createLoadable([]);
    this.sharedPayloads = createLoadable([]);
  }

  @LatestFlow("project")
  public *openProject(path: string): TFlow {
    const timer: Timer = new Timer();

    this.log.info("Opening archives project:", path);

    try {
      this.clearFileSelection();
      this.project = createLoadable(null, true);
      this.collisions = createLoadable([]);
      this.sharedPayloads = createLoadable([]);

      const response: ArchiveProject = yield* call(archivesCommands.openProject(path));

      this.log.info("Archives project opened in:", formatDuration(timer.elapsed()));

      this.project = createLoadable(response, false);

      yield* this.loadCollisions();
      yield* this.loadSharedPayloads();
    } catch (error: unknown) {
      this.log.error("Failed to open archives project after:", formatDuration(timer.elapsed()), error);

      this.project = createLoadable(null, false, transformError(error));

      emitNotification(this.eventBus, {
        details: `${path}\n${transformError(error).message}`,
        severity: ENotificationSeverity.ERROR,
        source: EApplicationId.ARCHIVES_EXPLORER,
        title: "Could not open archives project",
      });
    }
  }

  @LatestFlow("project")
  public *closeProject(): TFlow {
    const timer: Timer = new Timer();

    this.log.info("Closing existing archives project");

    try {
      yield* call(archivesCommands.closeProject());

      // Closing the project takes the preview off screen, so the model it parked in the backend goes with it.
      releaseEditorProject(visualsCommands.closeModel);

      this.log.info("Archives project closed in:", formatDuration(timer.elapsed()));

      this.clearFileSelection();
      this.project = createLoadable(null);
      this.collisions = createLoadable([]);
      this.sharedPayloads = createLoadable([]);
    } catch (error: unknown) {
      this.log.error("Failed to close archives project after:", formatDuration(timer.elapsed()), error);

      throw transformError(error);
    }
  }

  /**
   * Loads what the open volume set cannot reach, inside whichever flow opened it.
   *
   * Undecorated on purpose: it belongs to the open that asked for it, so a superseding open cancels this with the rest
   * of its own work instead of racing it from a lane of its own.
   */
  private *loadCollisions(): TFlow {
    try {
      this.collisions = createLoadable([], true);

      const collisions: Array<XrayPathCollision> = yield* call(archivesCommands.listCollisions());

      this.log.info("Archives project unreachable entries:", collisions.length);

      this.collisions = createLoadable(collisions, false);
    } catch (error: unknown) {
      this.log.error("Failed to list archives project collisions:", error);

      this.collisions = createLoadable([], false, transformError(error));
    }
  }

  /**
   * Loads which entries of the open volume set read the same bytes, inside whichever flow opened it.
   *
   * Undecorated for the same reason as the collisions: it belongs to the open that asked for it.
   */
  private *loadSharedPayloads(): TFlow {
    try {
      this.sharedPayloads = createLoadable([], true);

      const payloads: Array<ArchiveSharedPayload> = yield* call(archivesCommands.listSharedPayloads());

      this.log.info("Archives project shared payloads:", payloads.length);

      this.sharedPayloads = createLoadable(payloads, false);
    } catch (error: unknown) {
      this.log.error("Failed to list archives project shared payloads:", error);

      this.sharedPayloads = createLoadable([], false, transformError(error));
    }
  }

  @LatestFlow("content")
  public *selectArchiveFile(descriptor: ArchiveFileDescriptor): TFlow {
    this.log.info("Select archive file:", descriptor);

    this.selection = { kind: "file", descriptor };
    this.content = createLoadable(null);

    yield* this.loadSelectedContent(descriptor);
  }

  /**
   * Selects an archive directory instead of a file.
   *
   * @param path - Archive-relative directory path; an empty string selects the archive root.
   */
  @BoundAction()
  public selectArchiveDirectory(path: string): void {
    cancelFlow(this, "content");

    this.selection = { kind: "directory", path };
    this.content = createLoadable(null);
    this.operation = createLoadable(null);
  }

  @LatestFlow("content")
  public *retrySelectedFile(): TFlow {
    const descriptor: Nullable<ArchiveFileDescriptor> = this.selectedFile;

    if (descriptor) {
      yield* this.loadSelectedContent(descriptor);
    }
  }

  /**
   * Extracts an archived file to a destination path.
   *
   * @param descriptor - Archived file to extract.
   * @param destination - Output file path.
   * @returns Resolves after the extraction outcome is published.
   */
  @LatestFlow("operation")
  public *extractFile(descriptor: ArchiveFileDescriptor, destination: string): TFlow {
    const timer: Timer = new Timer();

    this.log.info("Extracting archive file:", descriptor.name, destination);

    try {
      this.operation = createLoadable(null, true);

      yield* call(archivesCommands.extractFile(descriptor.name, destination));

      this.log.info("Archive file extracted in:", formatDuration(timer.elapsed()));

      this.operation = createLoadable({ kind: "extract-file", destination });

      emitNotification(this.eventBus, {
        details: destination,
        severity: ENotificationSeverity.SUCCESS,
        source: EApplicationId.ARCHIVES_EXPLORER,
        title: `Extracted ${descriptor.name}`,
      });
    } catch (error: unknown) {
      this.log.error("Failed to extract archive file after:", formatDuration(timer.elapsed()), error);

      this.operation = createLoadable(null, false, transformError(error));

      emitNotification(this.eventBus, {
        details: `${destination}\n${transformError(error).message}`,
        severity: ENotificationSeverity.ERROR,
        source: EApplicationId.ARCHIVES_EXPLORER,
        title: `Could not extract ${descriptor.name}`,
      });

      throw transformError(error);
    }
  }

  /**
   * Extracts files beneath an archive directory into a destination root. An empty prefix extracts the archive root.
   *
   * @param prefix - Archive-relative directory prefix; an empty string selects the archive root.
   * @param destination - Output directory path.
   */
  @LatestFlow("operation")
  public *extractArchiveDirectory(prefix: string, destination: string): TFlow {
    const timer: Timer = new Timer();

    this.log.info("Extracting archive directory:", prefix || "<root>", destination);

    try {
      this.operation = createLoadable(null, true);

      // Started through the jobs service rather than invoked here: an empty prefix extracts the whole archive, so this
      // writes as much as an unpack does and wants the same identity, lease, and cancel control.
      const run: IJobRun<ArchiveExtractDirectoryResult> = this.jobsService.run<ArchiveExtractDirectoryResult>({
        kind: EJobKind.ARCHIVES_EXTRACT,
        invoke: (id: string, progress) => archivesCommands.extractDirectory(prefix, destination, id, progress),
        describe: (outcome: IJobOutcome<ArchiveExtractDirectoryResult>): IJobNotice =>
          describeExtractOutcome(prefix, destination, outcome),
      });

      this.jobId = run.id;

      const result: ArchiveExtractDirectoryResult = yield* call(run.promise);

      this.log.info("Archive directory extracted in:", formatDuration(timer.elapsed()));

      this.operation = createLoadable({ kind: "extract-directory", result });
    } catch (error: unknown) {
      this.log.error("Failed to extract archive directory after:", formatDuration(timer.elapsed()), error);

      this.operation = createLoadable(null, false, transformError(error));

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
    this.operation = createLoadable(null);
  }

  @BoundAction()
  public clearFileSelection(): void {
    cancelFlow(this, "content");

    this.selection = { kind: "none" };
    this.content = createLoadable(null);
    this.operation = createLoadable(null);
  }

  /**
   * Loads a selected file in its supported preview representation.
   *
   * A generator so a selection moved past is abandoned here too, rather than reading on and publishing over whatever
   * replaced it.
   *
   * @param descriptor - Selected archive file to preview.
   */
  private *loadSelectedContent(descriptor: ArchiveFileDescriptor): TFlow {
    const project: Nullable<ArchiveProject> = this.project.value;

    if (!project) {
      return;
    }

    // todo: Switch case based on type?
    if (isArchiveAudio(descriptor, project.readPolicy)) {
      return yield* this.readContent(descriptor, "audio", project);
    } else if (isArchiveImage(descriptor, project.readPolicy)) {
      return yield* this.readContent(descriptor, "image", project);
    } else if (getArchivePreviewSupport(descriptor, project.readPolicy).kind === "supported") {
      return yield* this.readContent(descriptor, "text", project);
    }
  }

  /**
   * Reads a sound as the description the engine would read plus the bytes the webview plays.
   *
   * Both calls name the same roots and the same logical path, so a late response cannot pair one file's numbers with
   * another file's sound. In parallel because neither needs the other.
   *
   * @param descriptor - Archive entry naming the sound.
   * @param project - Open project whose tree the sound is read out of.
   * @returns The sound's description and its bytes as stored.
   */
  private async readAudioContent(descriptor: ArchiveFileDescriptor, project: ArchiveProject): Promise<TArchiveContent> {
    const roots: XrayRoots = createArchiveRoots(project);

    const [audio, bytes] = await Promise.all([
      archivesCommands.describeAudio(roots, descriptor.name),
      assetsRawCommands.readAsset(roots, descriptor.name),
    ]);

    return { kind: "audio", descriptor: audio, bytes: new Uint8Array(bytes) };
  }

  /**
   * Reads a texture as its source shape plus the png the backend decoded it into.
   *
   * The description answers for the DDS and the bytes for the picture, which is why the read is domain owned rather
   * than the generic one the sound uses: the webview cannot paint a DDS.
   *
   * @param descriptor - Archive entry naming the texture.
   * @param project - Open project whose tree the texture is read out of.
   * @returns The texture's shape and the decoded png bytes.
   */
  private async readImageContent(descriptor: ArchiveFileDescriptor, project: ArchiveProject): Promise<TArchiveContent> {
    const roots: XrayRoots = createArchiveRoots(project);

    const [texture, bytes] = await Promise.all([
      archivesCommands.describeImage(roots, descriptor.name),
      archivesRawCommands.readImage(roots, descriptor.name),
    ]);

    return { kind: "image", descriptor: texture, bytes: new Uint8Array(bytes) };
  }

  /**
   * Loads and publishes one file preview.
   *
   * No staleness check of its own: a selection moved past cancels this where it stands, so the publish below the yield
   * cannot run for a file the explorer no longer points at.
   *
   * @param descriptor - Archive file to read.
   * @param kind - Preview representation to request from the backend.
   * @param project - Target project to read from.
   */
  private *readContent(
    descriptor: ArchiveFileDescriptor,
    kind: TArchiveContent["kind"],
    project: ArchiveProject
  ): TFlow {
    const timer: Timer = new Timer();

    this.log.info("Reading archive content:", kind, descriptor.name);
    this.content = createLoadable(null, true);

    try {
      const content: TArchiveContent = yield* call(
        kind === "audio"
          ? this.readAudioContent(descriptor, project)
          : kind === "image"
            ? this.readImageContent(descriptor, project)
            : archivesCommands.readFile(descriptor.name).then((result): TArchiveContent => ({ kind: "text", result }))
      );

      this.log.info("Archive content read in:", formatDuration(timer.elapsed()));

      this.content = createLoadable(content);
    } catch (error: unknown) {
      this.log.error("Failed to read archive content after:", formatDuration(timer.elapsed()), descriptor.name, error);

      this.content = createLoadable(null, false, transformError(error));
    }
  }
}
