import {
  EventBus,
  inject,
  Injectable,
  OnDeactivation,
  OnDeprovision,
  OnProvision,
  ProvisionId,
  WireStatus,
} from "@wirestate/core";
import { BoundAction, Computed, makeObservable, Observable, runInAction } from "@wirestate/mobx";

import {
  getArchivePreviewSupport,
  isArchiveAudio,
  isArchiveImage,
  listArchiveFiles,
  TArchiveContent,
  TArchiveOperation,
  TArchiveSelection,
} from "@/core/archive";
import { archivesCommands } from "@/core/bindings/commands/archives";
import { ArchiveFileDescriptor, ArchiveProject } from "@/core/bindings/types/xrf-archive";
import { ArchiveExtractDirectoryResult } from "@/core/bindings/types/xrf-pack";
import { transformError } from "@/core/error/lib";
import { releaseEditorProject } from "@/core/ipc/release";
import { emitNotification, ENotificationSeverity } from "@/core/notifications/lib";
import { EApplicationId } from "@/core/routing/application";
import { formatDuration } from "@/lib/format/duration";
import { createLoadable, Loadable } from "@/lib/loadable";
import { Logger, Timer } from "@/lib/logging";
import { Nullable } from "@/lib/types/general";

@Injectable()
export class ArchivesService {
  public readonly log: Logger = new Logger(this.constructor.name);

  private contentRequestId: number = 0;

  @Observable()
  public isReady: boolean = false;

  @Observable()
  public project: Loadable<Nullable<ArchiveProject>> = createLoadable(null);

  /** What the explorer points at. Exactly one kind at a time, by construction. */
  @Observable()
  public selection: TArchiveSelection = { kind: "none" };

  /** Whatever was loaded for the selection - text, a decoded texture, and later audio. */
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
   * Reports whether a content or write operation is in progress.
   *
   * @returns Whether navigation or another command would race with the active operation.
   */
  @Computed()
  public get isBusy(): boolean {
    return this.content.isLoading || this.operation.isLoading;
  }

  public constructor(
    private readonly status: WireStatus = WireStatus.track(this),
    private readonly eventBus: EventBus = inject(EventBus)
  ) {
    makeObservable(this);
  }

  @OnProvision()
  public async onProvision(provisionId: ProvisionId): Promise<void> {
    this.log.info("Provisioning:", provisionId);

    const existing: Nullable<ArchiveProject> = await archivesCommands.getProject();

    if (this.status.provisionId !== provisionId) {
      return this.log.info("Discard outdated get archives request:", provisionId, "<", this.status.provisionId);
    }

    if (existing) {
      this.log.info("Existing archives project detected");

      runInAction(() => {
        this.project = createLoadable(existing);
        this.isReady = true;
      });
    } else {
      this.log.info("No existing archives project");

      runInAction(() => {
        this.isReady = true;
      });
    }
  }

  @OnDeprovision()
  public onDeprovision(provisionId: ProvisionId): void {
    this.log.info("Deprovisioning:", provisionId);
  }

  /**
   * Releases the archive project when the editor deactivates.
   */
  @OnDeactivation()
  public onDeactivation(): void {
    this.log.info("Deactivating, release archive project");

    releaseEditorProject(archivesCommands.closeProject);
  }

  @BoundAction()
  public resetArchivesProject(): void {
    this.log.info("Reset archives project");

    this.clearFileSelection();
    this.project = createLoadable(null);
  }

  @BoundAction()
  public async openProject(path: string): Promise<void> {
    const timer: Timer = new Timer();

    this.log.info("Opening archives project:", path);

    try {
      this.clearFileSelection();
      this.project = createLoadable(null, true);

      const response: ArchiveProject = await archivesCommands.openProject(path);

      this.log.info("Archives project opened in:", formatDuration(timer.elapsed()));

      runInAction(() => (this.project = createLoadable(response, false)));
    } catch (error: unknown) {
      this.log.error("Failed to open archives project after:", formatDuration(timer.elapsed()), error);

      runInAction(() => (this.project = createLoadable(null, false, transformError(error))));

      emitNotification(this.eventBus, {
        details: `${path}\n${transformError(error).message}`,
        severity: ENotificationSeverity.ERROR,
        source: EApplicationId.ARCHIVES_EXPLORER,
        title: "Could not open archives project",
      });
    }
  }

  @BoundAction()
  public async closeProject(): Promise<void> {
    const timer: Timer = new Timer();

    this.log.info("Closing existing archives project");

    try {
      await archivesCommands.closeProject();

      this.log.info("Archives project closed in:", formatDuration(timer.elapsed()));

      runInAction(() => {
        this.clearFileSelection();
        this.project = createLoadable(null);
      });
    } catch (error: unknown) {
      this.log.error("Failed to close archives project after:", formatDuration(timer.elapsed()), error);

      throw transformError(error);
    }
  }

  @BoundAction()
  public async selectArchiveFile(descriptor: ArchiveFileDescriptor): Promise<void> {
    this.log.info("Select archive file:", descriptor);

    this.selection = { kind: "file", descriptor };
    this.contentRequestId += 1;
    this.content = createLoadable(null);

    await this.loadSelectedContent(descriptor);
  }

  /**
   * Selects an archive directory instead of a file.
   *
   * @param path - Archive-relative directory path; an empty string selects the archive root.
   */
  @BoundAction()
  public selectArchiveDirectory(path: string): void {
    this.contentRequestId += 1;
    this.selection = { kind: "directory", path };
    this.content = createLoadable(null);
    this.operation = createLoadable(null);
  }

  @BoundAction()
  public async retrySelectedFile(): Promise<void> {
    const descriptor: Nullable<ArchiveFileDescriptor> = this.selectedFile;

    if (descriptor) {
      await this.loadSelectedContent(descriptor);
    }
  }

  /**
   * Extracts an archived file to a destination path.
   *
   * @param descriptor - Archived file to extract.
   * @param destination - Output file path.
   * @returns Resolves after the extraction outcome is published.
   */
  @BoundAction()
  public async extractFile(descriptor: ArchiveFileDescriptor, destination: string): Promise<void> {
    const timer: Timer = new Timer();

    this.log.info("Extracting archive file:", descriptor.name, destination);

    try {
      this.operation = createLoadable(null, true);

      await archivesCommands.extractFile(descriptor.name, destination);

      this.log.info("Archive file extracted in:", formatDuration(timer.elapsed()));

      runInAction(() => (this.operation = createLoadable({ kind: "extract-file", destination })));

      emitNotification(this.eventBus, {
        details: destination,
        severity: ENotificationSeverity.SUCCESS,
        source: EApplicationId.ARCHIVES_EXPLORER,
        title: `Extracted ${descriptor.name}`,
      });
    } catch (error: unknown) {
      this.log.error("Failed to extract archive file after:", formatDuration(timer.elapsed()), error);

      runInAction(() => (this.operation = createLoadable(null, false, transformError(error))));

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
   * @returns Resolves after the extraction outcome is published.
   */
  @BoundAction()
  public async extractArchiveDirectory(prefix: string, destination: string): Promise<void> {
    const timer: Timer = new Timer();

    this.log.info("Extracting archive directory:", prefix || "<root>", destination);

    try {
      this.operation = createLoadable(null, true);

      const result: ArchiveExtractDirectoryResult = await archivesCommands.extractDirectory(prefix, destination);

      this.log.info("Archive directory extracted in:", formatDuration(timer.elapsed()));

      runInAction(() => (this.operation = createLoadable({ kind: "extract-directory", result })));

      // Reported without a count rather than not at all: a response the parser did not fill in is no
      // reason to turn a write that happened into a thrown error.
      const extractedCount: Nullable<number> = result?.extractedCount ?? null;
      const extractedFrom: string = prefix || "the archive root";

      emitNotification(this.eventBus, {
        details: destination,
        severity: ENotificationSeverity.SUCCESS,
        source: EApplicationId.ARCHIVES_EXPLORER,
        title:
          extractedCount === null
            ? `Extracted ${extractedFrom}`
            : `Extracted ${extractedCount} file(s) from ${extractedFrom}`,
      });
    } catch (error: unknown) {
      this.log.error("Failed to extract archive directory after:", formatDuration(timer.elapsed()), error);

      runInAction(() => (this.operation = createLoadable(null, false, transformError(error))));

      emitNotification(this.eventBus, {
        details: `${destination}\n${transformError(error).message}`,
        severity: ENotificationSeverity.ERROR,
        source: EApplicationId.ARCHIVES_EXPLORER,
        title: `Could not extract ${prefix || "the archive root"}`,
      });

      throw transformError(error);
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
    this.contentRequestId += 1;
    this.selection = { kind: "none" };
    this.content = createLoadable(null);
    this.operation = createLoadable(null);
  }

  /**
   * Loads a selected file in its supported preview representation.
   *
   * @param descriptor - Selected archive file to preview.
   * @returns Resolves after supported content loading is started or completed.
   */
  private async loadSelectedContent(descriptor: ArchiveFileDescriptor): Promise<void> {
    const project: Nullable<ArchiveProject> = this.project.value;

    if (!project) {
      return;
    }

    if (isArchiveAudio(descriptor, project.readPolicy)) {
      return await this.readContent(descriptor, "audio");
    }

    if (isArchiveImage(descriptor, project.readPolicy)) {
      return await this.readContent(descriptor, "image");
    }

    if (getArchivePreviewSupport(descriptor, project.readPolicy).kind === "supported") {
      return await this.readContent(descriptor, "text");
    }
  }

  /**
   * Loads and publishes one file preview while ignoring stale responses.
   *
   * @param descriptor - Archive file to read.
   * @param kind - Preview representation to request from the backend.
   * @returns Resolves after the current request publishes content or an error.
   */
  private async readContent(descriptor: ArchiveFileDescriptor, kind: TArchiveContent["kind"]): Promise<void> {
    const requestId: number = ++this.contentRequestId;
    const timer: Timer = new Timer();

    this.log.info("Reading archive content:", kind, descriptor.name);
    this.content = createLoadable(null, true);

    try {
      const content: TArchiveContent =
        kind === "audio"
          ? {
              kind: "audio",
              preview: await archivesCommands.readAudio(descriptor.name),
            }
          : kind === "image"
            ? {
                kind: "image",
                preview: await archivesCommands.readImage(descriptor.name),
              }
            : {
                kind: "text",
                result: await archivesCommands.readFile(descriptor.name),
              };

      if (requestId !== this.contentRequestId) {
        return;
      }

      this.log.info("Archive content read in:", formatDuration(timer.elapsed()));

      runInAction(() => (this.content = createLoadable(content)));
    } catch (error: unknown) {
      if (requestId !== this.contentRequestId) {
        return;
      }

      this.log.error("Failed to read archive content after:", formatDuration(timer.elapsed()), descriptor.name, error);

      runInAction(() => (this.content = createLoadable(null, false, transformError(error))));
    }
  }
}
