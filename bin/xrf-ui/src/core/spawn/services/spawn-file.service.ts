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

import { spawnCommands } from "@/core/bindings/commands/spawn";
import {
  SpawnALifeSpawnsChunk,
  SpawnArtefactSpawnsChunk,
  SpawnGraphsChunk,
  SpawnHeaderChunk,
  SpawnPatrolsChunk,
} from "@/core/bindings/types/xrf-db";
import { transformError } from "@/core/error/lib";
import { releaseEditorProject } from "@/core/ipc/release";
import { emitNotification, ENotificationSeverity } from "@/core/notifications/lib";
import { EApplicationGroupId } from "@/core/routing/application";
import { createLoadable, Loadable } from "@/lib/loadable";
import { Logger } from "@/lib/logging";
import { AnyObject, Nullable } from "@/lib/types/general";

export interface ISpawnRowSelection {
  /** What kind of row this is, for the panel heading. */
  source: string;
  id: string | number;
  row: AnyObject;
}

/**
 * The open spawn file, one chunk at a time.
 */
@Injectable()
export class SpawnFileService {
  public readonly log: Logger = new Logger(__MODULE_NAME__);

  @Observable()
  public isReady: boolean = false;

  /** Whether the backend holds an open file, known before any chunk has been read. */
  @Observable()
  public isOpen: boolean = false;

  /** Where the open file came from. Reported by the backend, so it survives a remount. */
  @Observable()
  public path: Nullable<string> = null;

  @Observable()
  public header: Loadable<Nullable<SpawnHeaderChunk>> = createLoadable(null);

  @Observable()
  public alifeSpawn: Loadable<Nullable<SpawnALifeSpawnsChunk>> = createLoadable(null);

  @Observable()
  public artefactSpawn: Loadable<Nullable<SpawnArtefactSpawnsChunk>> = createLoadable(null);

  @Observable()
  public patrols: Loadable<Nullable<SpawnPatrolsChunk>> = createLoadable(null);

  @Observable()
  public graphs: Loadable<Nullable<SpawnGraphsChunk>> = createLoadable(null);

  /** The last write to disk, so whichever surface started it can report the outcome. */
  @Observable()
  public operation: Loadable<Nullable<string>> = createLoadable(null);

  /**
   * The row the details panel is showing.
   */
  @Observable()
  public selectedRow: Nullable<ISpawnRowSelection> = null;

  /**
   * Whether something is in flight that a second command would race.
   *
   * Read by the editor to lock rail navigation and to disable its own commands. Derived here so the
   * toolbar, the rail and the forms cannot disagree about it.
   *
   * @returns Whether an operation is in progress.
   */
  @Computed()
  public get isBusy(): boolean {
    return (
      this.header.isLoading ||
      this.alifeSpawn.isLoading ||
      this.artefactSpawn.isLoading ||
      this.patrols.isLoading ||
      this.graphs.isLoading ||
      this.operation.isLoading
    );
  }

  public constructor(
    private readonly status: WireStatus = WireStatus.track(this),
    private readonly eventBus: EventBus = inject(EventBus)
  ) {
    makeObservable(this);
  }

  /**
   * Restore whatever the backend already had open.
   *
   * Asks whether a file is open before asking what is in it, so entering the editor with nothing open
   * costs one boolean instead of a parse.
   *
   * @param provisionId - Identifier for the current provisioning attempt.
   * @returns Completes after restoring the backend's open-file state.
   */
  @OnProvision()
  public async onProvision(provisionId: ProvisionId): Promise<void> {
    this.log.info("Provisioning:", provisionId);

    try {
      const isOpen: boolean = await spawnCommands.hasFile();

      this.log.info(isOpen ? "Existing spawn file detected" : "No existing spawn file");

      if (this.status.provisionId !== provisionId) {
        return this.log.info("Discard outdated init request:", provisionId, "<", this.status.provisionId);
      }

      runInAction(() => (this.isOpen = isOpen));

      if (isOpen) {
        await Promise.all([this.loadPath(), this.loadHeader()]);
      }
    } catch (error: unknown) {
      this.log.error("Failed to check for an existing spawn file:", error);
    } finally {
      // Always reached: leaving `isReady` false parks the editor on a spinner for the rest of the
      // session, with no way back to the open form.
      runInAction(() => (this.isReady = true));
    }
  }

  @OnDeprovision()
  public onDeprovision(provisionId: ProvisionId): void {
    this.log.info("Deprovisioning:", provisionId);
  }

  /**
   * Release the spawn file when the editor is navigated away from.
   */
  @OnDeactivation()
  public onDeactivation(): void {
    this.log.info("Deactivating");

    releaseEditorProject(spawnCommands.closeFile);
  }

  @BoundAction()
  public async openFile(path: string): Promise<void> {
    this.log.info("Opening spawn file:", path);

    this.resetChunks();
    this.header = createLoadable(null, true);

    try {
      const header: SpawnHeaderChunk = await spawnCommands.openFile(path);

      this.log.info("Spawn file opened");

      runInAction(() => {
        this.header = createLoadable(header);
        this.isOpen = true;
        this.path = path;
      });
    } catch (error: unknown) {
      this.log.error("Failed to open spawn file:", error);

      runInAction(() => {
        this.header = createLoadable(null, false, transformError(error));
        this.isOpen = false;
        this.path = null;
      });

      emitNotification(this.eventBus, {
        details: `${path}\n${transformError(error).message}`,
        severity: ENotificationSeverity.ERROR,
        source: EApplicationGroupId.SPAWNS,
        title: "Could not open spawn file",
      });
    }
  }

  @BoundAction()
  public async closeFile(): Promise<void> {
    this.log.info("Closing existing spawn file");

    try {
      await spawnCommands.closeFile();

      runInAction(() => {
        this.isOpen = false;
        this.path = null;
        this.header = createLoadable(null);
        this.operation = createLoadable(null);
        this.resetChunks();
      });
    } catch (error: unknown) {
      this.log.error("Failed to close spawn file:", error);

      emitNotification(this.eventBus, {
        details: transformError(error).message,
        severity: ENotificationSeverity.ERROR,
        source: EApplicationGroupId.SPAWNS,
        title: "Could not close spawn file",
      });
    }
  }

  @BoundAction()
  public async saveFile(path: string): Promise<void> {
    this.log.info("Saving spawn file:", path);

    this.operation = createLoadable(null, true);

    try {
      await spawnCommands.saveFile(path);

      runInAction(() => (this.operation = createLoadable("save")));

      emitNotification(this.eventBus, {
        details: path,
        severity: ENotificationSeverity.SUCCESS,
        source: EApplicationGroupId.SPAWNS,
        title: "Saved spawn file",
      });
    } catch (error: unknown) {
      this.log.error("Failed to save spawn file:", error);

      runInAction(() => (this.operation = createLoadable(null, false, transformError(error))));

      emitNotification(this.eventBus, {
        details: `${path}\n${transformError(error).message}`,
        severity: ENotificationSeverity.ERROR,
        source: EApplicationGroupId.SPAWNS,
        title: "Could not save spawn file",
      });
    }
  }

  @BoundAction()
  public async saveUnpackedDirectory(path: string): Promise<void> {
    this.log.info("Exporting spawn file:", path);

    this.operation = createLoadable(null, true);

    try {
      await spawnCommands.saveUnpackedDirectory(path);

      runInAction(() => (this.operation = createLoadable("export")));

      emitNotification(this.eventBus, {
        details: path,
        severity: ENotificationSeverity.SUCCESS,
        source: EApplicationGroupId.SPAWNS,
        title: "Exported spawn file",
      });
    } catch (error: unknown) {
      this.log.error("Failed to export spawn file:", error);

      runInAction(() => (this.operation = createLoadable(null, false, transformError(error))));

      emitNotification(this.eventBus, {
        details: `${path}\n${transformError(error).message}`,
        severity: ENotificationSeverity.ERROR,
        source: EApplicationGroupId.SPAWNS,
        title: "Could not export spawn file",
      });
    }
  }

  /**
   * Dismisses the last reported write outcome.
   */
  @BoundAction()
  public clearOperation(): void {
    this.operation = createLoadable(null);
  }

  @BoundAction()
  public selectRow(source: string, id: string | number, row: AnyObject): void {
    this.selectedRow = { id, row, source };
  }

  @BoundAction()
  public clearSelectedRow(): void {
    this.selectedRow = null;
  }

  @BoundAction()
  public async loadAlifeSpawn(): Promise<void> {
    await this.loadChunk("alifeSpawn", spawnCommands.getAlifeSpawns);
  }

  @BoundAction()
  public async loadArtefactSpawn(): Promise<void> {
    await this.loadChunk("artefactSpawn", spawnCommands.getArtefactSpawns);
  }

  @BoundAction()
  public async loadPatrols(): Promise<void> {
    await this.loadChunk("patrols", spawnCommands.getPatrols);
  }

  @BoundAction()
  public async loadGraphs(): Promise<void> {
    await this.loadChunk("graphs", spawnCommands.getGraphs);
  }

  @BoundAction()
  public async loadHeader(): Promise<void> {
    await this.loadChunk("header", spawnCommands.getHeader);
  }

  /**
   * Fetch one chunk, at most once.
   *
   * Views ask for their chunk on mount, and moving between chunk tabs remounts them, so without the
   * already-loaded guard every tab click would refetch what it is about to render.
   *
   * @param key - State field that stores the requested chunk.
   * @param request - Backend command that loads the chunk.
   * @returns Completes after the chunk state is already available or updated.
   */
  private async loadChunk<K extends "header" | "alifeSpawn" | "artefactSpawn" | "patrols" | "graphs">(
    key: K,
    request: () => Promise<unknown>
  ): Promise<void> {
    const current: Loadable<unknown> = this[key];

    // Deliberately not gated on `isOpen`: that is set asynchronously while provisioning, so a chunk view
    // mounting first would load nothing and never retry. The backend answers null when nothing is open,
    // which is the same empty state by a shorter route.
    if (current.isLoading || current.value !== null) {
      return;
    }

    runInAction(() => ((this[key] as Loadable<unknown>) = createLoadable(null, true)));

    try {
      const chunk: unknown = await request();

      runInAction(() => ((this[key] as Loadable<unknown>) = createLoadable(chunk)));
    } catch (error: unknown) {
      this.log.error("Failed to read spawn chunk:", key, error);

      runInAction(() => ((this[key] as Loadable<unknown>) = createLoadable(null, false, transformError(error))));

      emitNotification(this.eventBus, {
        details: transformError(error).message,
        severity: ENotificationSeverity.ERROR,
        source: EApplicationGroupId.SPAWNS,
        title: `Could not read the ${key} chunk`,
      });
    }
  }

  private async loadPath(): Promise<void> {
    try {
      const path: Nullable<string> = await spawnCommands.getPath();

      runInAction(() => (this.path = path));
    } catch (error: unknown) {
      // The path only names what is open, so failing to read it must not look like a failure to open.
      this.log.error("Failed to read spawn file path:", error);
    }
  }

  private resetChunks(): void {
    this.alifeSpawn = createLoadable(null);
    this.artefactSpawn = createLoadable(null);
    this.patrols = createLoadable(null);
    this.graphs = createLoadable(null);
    // A selection outlives its table, so it has to be dropped with the data it pointed into.
    this.selectedRow = null;
  }
}
