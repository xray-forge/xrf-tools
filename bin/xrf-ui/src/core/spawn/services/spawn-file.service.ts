import { EventBus, inject, Injectable, OnDeactivation, OnDeprovision, OnProvision, ProvisionId } from "@wirestate/core";
import { BoundAction, Computed, flowResult, Observable } from "@wirestate/mobx";

import { spawnCommands } from "@/core/bindings/commands/spawn";
import { SessionId, SpawnSessionDescriptor } from "@/core/bindings/types/xrf-app";
import { SpawnFile } from "@/core/bindings/types/xrf-db";
import { transformError } from "@/core/error/lib";
import { releaseEditorProject } from "@/core/ipc/release";
import { Session } from "@/core/ipc/session";
import { emitNotification, ENotificationSeverity } from "@/core/notifications/lib";
import { EApplicationGroupId } from "@/core/routing/application";
import { AsyncState } from "@/lib/async-state";
import { Logger } from "@/lib/logging";
import { call, cancelFlow, ExclusiveFlow, LatestFlow, TFlow } from "@/lib/mobx";
import { AnyObject, Nullable } from "@/lib/types/general";

export interface ISpawnRowSelection {
  /** What kind of row this is, for the panel heading. */
  source: string;
  id: string | number;
  row: AnyObject;
}

type TSpawnChunkStates = { readonly [K in keyof SpawnFile]: AsyncState<SpawnFile[K]> };

@Injectable()
export class SpawnFileService {
  public readonly log: Logger = new Logger(__MODULE_NAME__);

  private readonly session: Session = new Session(spawnCommands.closeFile);

  @Observable()
  public isReady: boolean = false;

  @Observable()
  private sessionDescriptor: Nullable<SpawnSessionDescriptor> = null;

  @Observable()
  public isOpening: boolean = false;

  @Observable()
  public chunks: TSpawnChunkStates = {
    header: AsyncState.idle(),
    alifeSpawn: AsyncState.idle(),
    artefactSpawn: AsyncState.idle(),
    patrols: AsyncState.idle(),
    graphs: AsyncState.idle(),
  };

  @Computed()
  public get sessionId(): Nullable<SessionId> {
    return this.sessionDescriptor?.sessionId ?? null;
  }

  @Computed()
  public get isOpen(): boolean {
    return this.sessionDescriptor !== null;
  }

  @Computed()
  public get path(): Nullable<string> {
    return this.sessionDescriptor?.path ?? null;
  }

  /** The last write to disk, so whichever surface started it can report the outcome. */
  @Observable()
  public operation: AsyncState<string> = AsyncState.idle();

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
      this.isOpening ||
      this.chunks.alifeSpawn.isLoading ||
      this.chunks.artefactSpawn.isLoading ||
      this.chunks.patrols.isLoading ||
      this.chunks.graphs.isLoading ||
      this.operation.isLoading
    );
  }

  public constructor(private readonly eventBus: EventBus = inject(EventBus)) {}

  /**
   * Restore whatever the backend already had open.
   *
   * Restores identity, path and header from one backend snapshot; heavy chunks remain lazy.
   *
   * @param provisionId - Identifier for the current provisioning attempt.
   * @returns Completes after restoring the backend's open-file state.
   */
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
   * Release the spawn file when the editor is navigated away from.
   */
  @OnDeactivation()
  public onDeactivation(): void {
    this.log.info("Deactivating");

    releaseEditorProject(() => this.session.close(this.sessionId));
  }

  /**
   * Restores the committed session without superseding a user action in the same flow.
   */
  @ExclusiveFlow("isOpening")
  private *restore(): TFlow {
    try {
      this.adoptSession(yield* call(spawnCommands.getSession()));
    } catch (error: unknown) {
      this.log.error("Failed to check for an existing spawn file:", error);
    } finally {
      // Always reached, cancellation included: leaving `isReady` false parks the editor on a spinner for the rest of
      // the session, with no way back to the open form.
      this.isReady = true;
    }
  }

  @LatestFlow("isOpening")
  public *openFile(path: string): TFlow {
    this.log.info("Opening spawn file:", path);

    this.isOpening = true;

    if (!this.sessionDescriptor) {
      this.setChunk("header", AsyncState.idle());
    }

    try {
      this.adoptSession(yield* call(this.session.open(spawnCommands.openFile, path)));
      this.log.info("Spawn file opened");
    } catch (error: unknown) {
      this.log.error("Failed to open spawn file:", error);

      if (!this.sessionDescriptor) {
        this.setChunk("header", this.chunks.header.asFailed(transformError(error), null));
      }

      emitNotification(this.eventBus, {
        details: `${path}\n${transformError(error).message}`,
        severity: ENotificationSeverity.ERROR,
        source: EApplicationGroupId.SPAWNS,
        title: "Could not open spawn file",
      });
    } finally {
      this.isOpening = false;
    }
  }

  @LatestFlow("isOpening")
  public *closeFile(): TFlow {
    this.log.info("Closing existing spawn file");

    try {
      yield* call(this.session.close(this.sessionId));

      this.adoptSession(null);
      this.operation = this.operation.asIdle();
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

  @LatestFlow("operation")
  public *saveFile(path: string): TFlow {
    const session = this.sessionDescriptor;

    if (!session) {
      return;
    }

    this.log.info("Saving spawn file:", path);

    this.operation = this.operation.asLoading(null);

    try {
      yield* call(spawnCommands.saveFile(path, session.sessionId));

      this.operation = this.operation.asReady("save");

      emitNotification(this.eventBus, {
        details: path,
        severity: ENotificationSeverity.SUCCESS,
        source: EApplicationGroupId.SPAWNS,
        title: "Saved spawn file",
      });
    } catch (error: unknown) {
      this.log.error("Failed to save spawn file:", error);

      this.operation = this.operation.asFailed(transformError(error), null);

      emitNotification(this.eventBus, {
        details: `${path}\n${transformError(error).message}`,
        severity: ENotificationSeverity.ERROR,
        source: EApplicationGroupId.SPAWNS,
        title: "Could not save spawn file",
      });
    }
  }

  @LatestFlow("operation")
  public *saveUnpackedDirectory(path: string): TFlow {
    const session = this.sessionDescriptor;

    if (!session) {
      return;
    }

    this.log.info("Exporting spawn file:", path);

    this.operation = this.operation.asLoading(null);

    try {
      yield* call(spawnCommands.saveUnpackedDirectory(path, session.sessionId));

      this.operation = this.operation.asReady("export");

      emitNotification(this.eventBus, {
        details: path,
        severity: ENotificationSeverity.SUCCESS,
        source: EApplicationGroupId.SPAWNS,
        title: "Exported spawn file",
      });
    } catch (error: unknown) {
      this.log.error("Failed to export spawn file:", error);

      this.operation = this.operation.asFailed(transformError(error), null);

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
    this.operation = this.operation.asIdle();
  }

  @BoundAction()
  public selectRow(source: string, id: string | number, row: AnyObject): void {
    this.selectedRow = { id, row, source };
  }

  @BoundAction()
  public clearSelectedRow(): void {
    this.selectedRow = null;
  }

  @ExclusiveFlow()
  public *loadAlifeSpawn(): TFlow {
    yield* this.fetchChunk("alifeSpawn", spawnCommands.getAlifeSpawns);
  }

  @ExclusiveFlow()
  public *loadArtefactSpawn(): TFlow {
    yield* this.fetchChunk("artefactSpawn", spawnCommands.getArtefactSpawns);
  }

  @ExclusiveFlow()
  public *loadPatrols(): TFlow {
    yield* this.fetchChunk("patrols", spawnCommands.getPatrols);
  }

  @ExclusiveFlow()
  public *loadGraphs(): TFlow {
    yield* this.fetchChunk("graphs", spawnCommands.getGraphs);
  }

  /**
   * Fetch one chunk, at most once.
   *
   * Views ask for their chunk on mount, and moving between chunk tabs remounts them, so without the
   * already-loaded guard every tab click would refetch what it is about to render.
   *
   * A generator rather than an async method, so a chunk abandoned by a reset is abandoned here too: cancelling the
   * caller resumes this with a return completion, and the write below the yield never happens.
   *
   * @param key - State field that stores the requested chunk.
   * @param read - Command returning that chunk for the committed session.
   */
  private *fetchChunk<K extends keyof SpawnFile>(key: K, read: (sessionId: SessionId) => Promise<SpawnFile[K]>): TFlow {
    const session = this.sessionDescriptor;
    const current: AsyncState<SpawnFile[K]> = this.chunks[key];

    if (!session || current.isLoading || current.isReady) {
      return;
    }

    const loading = current.asLoading(null);

    this.setChunk(key, loading);

    try {
      const chunk: SpawnFile[K] = yield* call(read(session.sessionId));

      this.setChunk(key, loading.asReady(chunk));
    } catch (error: unknown) {
      this.log.error("Failed to read spawn chunk:", key, error);

      this.setChunk(key, loading.asFailed(transformError(error)));

      emitNotification(this.eventBus, {
        details: transformError(error).message,
        severity: ENotificationSeverity.ERROR,
        source: EApplicationGroupId.SPAWNS,
        title: `Could not read the ${key} chunk`,
      });
    } finally {
      // Cancelling a read leaves it retryable without replacing a newer state.
      if (this.chunks[key] === loading) {
        this.setChunk(key, current);
      }
    }
  }

  private setChunk<K extends keyof SpawnFile>(key: K, value: AsyncState<SpawnFile[K]>): void {
    this.chunks = { ...this.chunks, [key]: value };
  }

  private adoptSession(session: Nullable<SpawnSessionDescriptor>): void {
    if (!session || this.sessionDescriptor?.sessionId !== session.sessionId) {
      this.resetChunks();
    }

    this.sessionDescriptor = session;
    this.setChunk("header", session ? AsyncState.ready(session.header) : AsyncState.idle());
  }

  private resetChunks(): void {
    // Abandoned rather than merely cleared: a read already on the wire would otherwise land under the next file.
    for (const lane of ["loadAlifeSpawn", "loadArtefactSpawn", "loadPatrols", "loadGraphs"] as const) {
      cancelFlow(this, lane);
    }

    this.chunks = {
      header: AsyncState.idle(),
      alifeSpawn: AsyncState.idle(),
      artefactSpawn: AsyncState.idle(),
      patrols: AsyncState.idle(),
      graphs: AsyncState.idle(),
    };

    // A selection outlives its table, so it has to be dropped with the data it pointed into.
    this.selectedRow = null;
  }
}
