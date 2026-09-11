import { EventBus, inject, Injectable, OnDeactivation, OnDeprovision, OnProvision, ProvisionId } from "@wirestate/core";
import { BoundAction, Computed, flowResult, Observable } from "@wirestate/mobx";

import { spawnCommands } from "@/core/bindings/commands/spawn";
import { SpawnSessionDescriptor, SpawnSessionId } from "@/core/bindings/types/xrf-app";
import { SpawnFile } from "@/core/bindings/types/xrf-db";
import { transformError } from "@/core/error/lib";
import { releaseEditorProject } from "@/core/ipc/release";
import { emitNotification, ENotificationSeverity } from "@/core/notifications/lib";
import { EApplicationGroupId } from "@/core/routing/application";
import { Loadable } from "@/lib/loadable";
import { Logger } from "@/lib/logging";
import { call, cancelFlow, ExclusiveFlow, LatestFlow, TFlow } from "@/lib/mobx";
import { AnyObject, Nullable } from "@/lib/types/general";

export interface ISpawnRowSelection {
  /** What kind of row this is, for the panel heading. */
  source: string;
  id: string | number;
  row: AnyObject;
}

type TSpawnChunkStates = { readonly [K in keyof SpawnFile]: Loadable<SpawnFile[K]> };

@Injectable()
export class SpawnFileService {
  public readonly log: Logger = new Logger(__MODULE_NAME__);

  @Observable()
  public isReady: boolean = false;

  @Observable()
  private session: Nullable<SpawnSessionDescriptor> = null;

  @Observable()
  public isOpening: boolean = false;

  @Observable()
  private chunkStates: TSpawnChunkStates = {
    header: Loadable.idle(),
    alifeSpawn: Loadable.idle(),
    artefactSpawn: Loadable.idle(),
    patrols: Loadable.idle(),
    graphs: Loadable.idle(),
  };

  public get chunks(): TSpawnChunkStates {
    return this.chunkStates;
  }

  @Computed()
  public get sessionId(): Nullable<SpawnSessionId> {
    return this.session?.id ?? null;
  }

  @Computed()
  public get isOpen(): boolean {
    return this.session !== null;
  }

  @Computed()
  public get path(): Nullable<string> {
    return this.session?.path ?? null;
  }

  /** The last write to disk, so whichever surface started it can report the outcome. */
  @Observable()
  public operation: Loadable<Nullable<string>> = Loadable.idle(null);

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

    releaseEditorProject(spawnCommands.closeFile);
  }

  /**
   * Puts back whatever the backend already had open.
   *
   * Exclusive rather than latest. A restore must lose to anything the user started: joining the lane
   * leaves an open in progress alone, where superseding would cancel the very thing the user asked for. The user's
   * own actions take the lane the other way round, so an open cancels a restore that is still in flight.
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

    if (!this.session) {
      this.setChunk("header", Loadable.idle());
    }

    try {
      this.adoptSession(yield* call(spawnCommands.openFile(path)));
      this.log.info("Spawn file opened");
    } catch (error: unknown) {
      this.log.error("Failed to open spawn file:", error);

      // An earlier open may have committed after its frontend flow was cancelled.
      try {
        this.adoptSession(yield* call(spawnCommands.getSession()));
      } catch (restoreError: unknown) {
        this.log.error("Failed to restore the spawn session:", restoreError);
      }

      if (!this.session) {
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
      yield* call(spawnCommands.closeFile());

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
    const session = this.session;

    if (!session) {
      return;
    }

    this.log.info("Saving spawn file:", path);

    this.operation = this.operation.asLoading(null);

    try {
      yield* call(spawnCommands.saveFile(path, session.id));

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
    const session = this.session;

    if (!session) {
      return;
    }

    this.log.info("Exporting spawn file:", path);

    this.operation = this.operation.asLoading(null);

    try {
      yield* call(spawnCommands.saveUnpackedDirectory(path, session.id));

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
  private *fetchChunk<K extends keyof SpawnFile>(
    key: K,
    read: (sessionId: SpawnSessionId) => Promise<SpawnFile[K]>
  ): TFlow {
    const session = this.session;
    const current: Loadable<SpawnFile[K]> = this.chunks[key];

    if (!session || current.isLoading || current.isReady) {
      return;
    }

    const loading = current.asLoading(null);

    this.setChunk(key, loading);

    try {
      const chunk: SpawnFile[K] = yield* call(read(session.id));

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

  private setChunk<K extends keyof SpawnFile>(key: K, value: Loadable<SpawnFile[K]>): void {
    this.chunkStates = { ...this.chunks, [key]: value };
  }

  private adoptSession(session: Nullable<SpawnSessionDescriptor>): void {
    if (!session || this.session?.id !== session.id) {
      this.resetChunks();
    }

    this.session = session;
    this.setChunk("header", session ? Loadable.ready(session.header) : Loadable.idle());
  }

  private resetChunks(): void {
    // Abandoned rather than merely cleared: a read already on the wire would otherwise land under the next file.
    for (const lane of ["loadAlifeSpawn", "loadArtefactSpawn", "loadPatrols", "loadGraphs"] as const) {
      cancelFlow(this, lane);
    }

    this.chunkStates = {
      header: Loadable.idle(),
      alifeSpawn: Loadable.idle(),
      artefactSpawn: Loadable.idle(),
      patrols: Loadable.idle(),
      graphs: Loadable.idle(),
    };

    // A selection outlives its table, so it has to be dropped with the data it pointed into.
    this.selectedRow = null;
  }
}
