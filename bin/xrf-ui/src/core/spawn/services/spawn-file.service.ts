import { EventBus, inject, Injectable, OnDeprovision, OnProvision, ProvisionId } from "@wirestate/core";
import { BoundAction, Computed, flowResult, Observable } from "@wirestate/mobx";
import { AsyncState } from "@xrf/async";
import { formatDuration } from "@xrf/format";
import { AnyObject, Nullable } from "@xrf/types";

import { transformError } from "@/core/error/lib";
import { spawnCommands } from "@/core/ipc/commands/spawn";
import { Session, SessionScopeService } from "@/core/ipc/session";
import {
  ESpawnChunkKind,
  SessionId,
  SessionSnapshot,
  SpawnChunk,
  SpawnSessionDescriptor,
} from "@/core/ipc/types/xrf-app";
import { SpawnHeaderChunk } from "@/core/ipc/types/xrf-spawn";
import { emitNotification, ENotificationSeverity } from "@/core/notifications/lib";
import { EApplicationGroupId } from "@/core/routing/application";
import { Logger, Timer } from "@/lib/logging";
import { call, cancelFlow, ExclusiveFlow, LatestFlow, TFlow } from "@/lib/mobx";

export interface ISpawnRowSelection {
  /** What kind of row this is, for the panel heading. */
  source: string;
  id: string | number;
  row: AnyObject;
}

/** What one large chunk reads as, by the kind it is asked for: `read_chunk` answers every kind as one tagged union. */
type TSpawnChunkOf<K extends ESpawnChunkKind> = Extract<SpawnChunk, { kind: `${K}` }>;

/** Each large chunk as it is read, by the kind it is read as. */
type TSpawnLargeChunkStates = { readonly [K in ESpawnChunkKind]: AsyncState<TSpawnChunkOf<K>> };

/** The header the open answers, and each large chunk as it is read. */
type TSpawnChunkStates = { readonly header: AsyncState<SpawnHeaderChunk> } & TSpawnLargeChunkStates;

/**
 * Whether a chunk read back is the kind that was asked for.
 *
 * @param chunk - What the read answered.
 * @param kind - What it was asked for.
 * @returns Whether the chunk is of that kind.
 */
function isSpawnChunkOf<K extends ESpawnChunkKind>(chunk: SpawnChunk, kind: K): chunk is TSpawnChunkOf<K> {
  return chunk.kind === kind;
}

@Injectable()
export class SpawnFileService {
  public readonly log: Logger = new Logger(__MODULE_NAME__);

  private readonly session: Session = new Session(spawnCommands.closeFile, inject(SessionScopeService));

  @Observable()
  public isReady: boolean = false;

  @Observable()
  private sessionState: Nullable<SessionSnapshot<SpawnSessionDescriptor>> = null;

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
    return this.sessionState?.sessionId ?? null;
  }

  @Computed()
  public get isOpen(): boolean {
    return this.sessionState !== null;
  }

  @Computed()
  public get path(): Nullable<string> {
    return this.sessionState?.value.path ?? null;
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
   * Restores the committed session without superseding a user action in the same flow.
   */
  @ExclusiveFlow("isOpening")
  private *restore(): TFlow {
    try {
      const snapshot: Nullable<SessionSnapshot<SpawnSessionDescriptor>> = yield* call(
        spawnCommands.restoreFile(this.session.tool)
      );

      this.adoptSession(snapshot);

      if (this.sessionState) {
        this.log.info("Spawn file restored:", this.path, this.sessionId);
      }
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
    const timer: Timer = new Timer();

    this.log.info("Opening spawn file:", path);

    this.isOpening = true;

    if (!this.sessionState) {
      this.setHeader(AsyncState.idle());
    }

    try {
      this.adoptSession(yield* call(this.session.open(spawnCommands.openFile, path)));
      this.log.info("Spawn file opened:", path, "in", formatDuration(timer.elapsed()));
    } catch (error: unknown) {
      this.log.error("Failed to open spawn file:", path, "after", formatDuration(timer.elapsed()), error);

      if (!this.sessionState) {
        this.setHeader(this.chunks.header.asFailed(transformError(error), null));
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
    const path: Nullable<string> = this.path;

    this.log.info("Closing existing spawn file:", path);

    try {
      yield* call(this.session.close());

      this.adoptSession(null);
      this.operation = this.operation.asIdle();

      this.log.info("Spawn file closed:", path);
    } catch (error: unknown) {
      this.log.error("Failed to close spawn file:", path, error);

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
    const session = this.sessionState;

    if (!session) {
      return;
    }

    const timer: Timer = new Timer();

    this.log.info("Saving spawn file:", path);

    this.operation = this.operation.asLoading(null);

    try {
      yield* call(spawnCommands.saveFile(path, session.sessionId));

      this.operation = this.operation.asReady("save");

      this.log.info("Spawn file saved:", path, "in", formatDuration(timer.elapsed()));

      emitNotification(this.eventBus, {
        details: path,
        severity: ENotificationSeverity.SUCCESS,
        source: EApplicationGroupId.SPAWNS,
        title: "Saved spawn file",
      });
    } catch (error: unknown) {
      this.log.error("Failed to save spawn file:", path, "after", formatDuration(timer.elapsed()), error);

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
    const session = this.sessionState;

    if (!session) {
      return;
    }

    const timer: Timer = new Timer();

    this.log.info("Exporting spawn file:", path);

    this.operation = this.operation.asLoading(null);

    try {
      yield* call(spawnCommands.saveUnpackedDirectory(path, session.sessionId));

      this.operation = this.operation.asReady("export");

      this.log.info("Spawn file exported:", path, "in", formatDuration(timer.elapsed()));

      emitNotification(this.eventBus, {
        details: path,
        severity: ENotificationSeverity.SUCCESS,
        source: EApplicationGroupId.SPAWNS,
        title: "Exported spawn file",
      });
    } catch (error: unknown) {
      this.log.error("Failed to export spawn file:", path, "after", formatDuration(timer.elapsed()), error);

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

  @ExclusiveFlow()
  public *loadAlifeSpawn(): TFlow {
    yield* this.fetchChunk(ESpawnChunkKind.ALIFE_SPAWN);
  }

  @ExclusiveFlow()
  public *loadArtefactSpawn(): TFlow {
    yield* this.fetchChunk(ESpawnChunkKind.ARTEFACT_SPAWN);
  }

  @ExclusiveFlow()
  public *loadPatrols(): TFlow {
    yield* this.fetchChunk(ESpawnChunkKind.PATROLS);
  }

  @ExclusiveFlow()
  public *loadGraphs(): TFlow {
    yield* this.fetchChunk(ESpawnChunkKind.GRAPHS);
  }

  /**
   * Fetch one chunk, at most once.
   *
   * @param key - The chunk to read, which also names the state field it is kept in.
   */
  private *fetchChunk<K extends ESpawnChunkKind>(key: K): TFlow {
    const session = this.sessionState;
    const chunks: TSpawnLargeChunkStates = this.chunks;
    const current: AsyncState<TSpawnChunkOf<K>> = chunks[key];

    if (!session || current.isLoading || current.isReady) {
      return;
    }

    const loading = current.asLoading(null);
    const timer: Timer = new Timer();
    const path: Nullable<string> = this.path;

    this.log.info("Reading spawn chunk:", path, key);
    this.setLargeChunk(key, loading);

    try {
      const answer: SessionSnapshot<SpawnChunk> = yield* call(spawnCommands.readChunk(session.sessionId, key));

      if (!isSpawnChunkOf(answer.value, key)) {
        throw new Error(`Asked for the ${key} chunk, read the ${answer.value.kind} chunk`);
      }

      this.setLargeChunk(key, loading.asReady(answer.value));

      this.log.info("Spawn chunk read:", path, key, "in", formatDuration(timer.elapsed()));
    } catch (error: unknown) {
      this.log.error("Failed to read spawn chunk:", path, key, "after", formatDuration(timer.elapsed()), error);

      this.setLargeChunk(key, loading.asFailed(transformError(error)));

      emitNotification(this.eventBus, {
        details: transformError(error).message,
        severity: ENotificationSeverity.ERROR,
        source: EApplicationGroupId.SPAWNS,
        title: `Could not read the ${key} chunk`,
      });
    } finally {
      // Cancelling a read leaves it retryable without replacing a newer state.
      if (this.chunks[key] === loading) {
        this.setLargeChunk(key, current);
      }
    }
  }

  private setHeader(value: AsyncState<SpawnHeaderChunk>): void {
    this.chunks = { ...this.chunks, header: value };
  }

  private setLargeChunk<K extends ESpawnChunkKind>(key: K, value: AsyncState<TSpawnChunkOf<K>>): void {
    this.chunks = { ...this.chunks, [key]: value };
  }

  private adoptSession(session: Nullable<SessionSnapshot<SpawnSessionDescriptor>>): void {
    if (!session || this.sessionState?.sessionId !== session.sessionId) {
      this.resetChunks();
    }

    this.session.adopt(session);
    this.sessionState = session;

    this.setHeader(session ? AsyncState.ready(session.value.header) : AsyncState.idle());
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
