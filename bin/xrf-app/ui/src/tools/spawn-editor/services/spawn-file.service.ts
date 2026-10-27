import { Injectable, OnProvision } from "@wirestate/core";
import { BoundAction, Computed, flowResult, Observable } from "@wirestate/mobx";
import { AsyncState } from "@xrf/async";
import { cancelFlow, ExclusiveFlow, LatestFlow, TFlow } from "@xrf/flow";
import { spawnCommands } from "@xrf/ipc/commands/spawn";
import { SessionId, SessionSnapshot } from "@xrf/ipc/types/xrf-app-core";
import { ESpawnChunkKind, SpawnChunk, SpawnSessionDescriptor } from "@xrf/ipc/types/xrf-app-spawn";
import { SpawnHeaderChunk } from "@xrf/ipc/types/xrf-spawn";
import { Logger } from "@xrf/logging";
import { AnyObject, Nullable } from "@xrf/types";

import { CommandOperation } from "@/core/operations/lib/command-operation";
import { SessionRootResource } from "@/core/session/lib/session-root-resource";

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

/** Each large chunk's read, by the kind it is read as. */
type TSpawnLargeChunkReads = { readonly [K in ESpawnChunkKind]: CommandOperation<TSpawnChunkOf<K>> };

/** The header the open answers, and each large chunk as it is read. */
type TSpawnChunkStates = { readonly header: AsyncState<SpawnHeaderChunk> } & TSpawnLargeChunkStates;

/** The lanes each large chunk is read on, which a new file abandons. */
const CHUNK_LANES = ["loadAlifeSpawn", "loadArtefactSpawn", "loadPatrols", "loadGraphs"] as const;

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

  /** The open spawn file, its header answered with it. */
  public readonly file: SessionRootResource<SpawnSessionDescriptor> = new SessionRootResource(this.log, {
    subject: "spawn file",
    close: spawnCommands.closeFile,
    restore: spawnCommands.restoreFile,
    adopted: () => this.forgetChunks(),
  });

  /** The last write to disk, so whichever surface started it can report the outcome. */
  public readonly operation: CommandOperation<string> = new CommandOperation(this.log);

  /**
   * The row the details panel is showing.
   */
  @Observable()
  public selectedRow: Nullable<ISpawnRowSelection> = null;

  /** Each large chunk, read once, on demand. */
  private readonly reads: TSpawnLargeChunkReads = {
    [ESpawnChunkKind.ALIFE_SPAWN]: new CommandOperation(this.log),
    [ESpawnChunkKind.ARTEFACT_SPAWN]: new CommandOperation(this.log),
    [ESpawnChunkKind.PATROLS]: new CommandOperation(this.log),
    [ESpawnChunkKind.GRAPHS]: new CommandOperation(this.log),
  };

  /** Whether a file is opening, closing or being restored. */
  @Computed()
  public get isOpening(): boolean {
    return this.file.isLoading;
  }

  /** The header the open answered, and each large chunk as it is read. */
  @Computed()
  public get chunks(): TSpawnChunkStates {
    return {
      header: this.file.state.map((file: SpawnSessionDescriptor) => file.header),
      alifeSpawn: this.reads.alifeSpawn.state,
      artefactSpawn: this.reads.artefactSpawn.state,
      patrols: this.reads.patrols.state,
      graphs: this.reads.graphs.state,
    };
  }

  @Computed()
  public get sessionId(): Nullable<SessionId> {
    return this.file.sessionId;
  }

  @Computed()
  public get isOpen(): boolean {
    return this.file.value !== null;
  }

  @Computed()
  public get path(): Nullable<string> {
    return this.file.value?.path ?? null;
  }

  /**
   * Whether something is in flight that a second command would race.
   *
   * @returns Whether an operation is in progress.
   */
  @Computed()
  public get isBusy(): boolean {
    return this.file.isLoading || Object.values(this.reads).some((read) => read.isLoading) || this.operation.isLoading;
  }

  /**
   * Restore whatever the backend already had open.
   */
  @OnProvision()
  public async onProvision(): Promise<void> {
    await flowResult(this.file.restore());
  }

  /**
   * @param path - The spawn file to open.
   * @returns Whether it opened.
   */
  @BoundAction()
  public openFile(path: string): Promise<boolean> {
    return flowResult(this.file.open(path, spawnCommands.openFile, path));
  }

  @BoundAction()
  public async closeFile(): Promise<void> {
    if (await flowResult(this.file.close())) {
      this.forgetChunks();
      this.operation.reset();
    }
  }

  @LatestFlow("operation")
  public *saveFile(path: string): TFlow {
    const sessionId: Nullable<SessionId> = this.sessionId;

    if (!sessionId) {
      return;
    }

    yield* this.operation.run({
      name: "Save spawn file",
      subject: path,
      command: () => spawnCommands.saveFile(path, sessionId).then(() => "save"),
      failure: "Could not save spawn file",
      success: () => ({ details: path, title: "Saved spawn file" }),
    });
  }

  @LatestFlow("operation")
  public *saveUnpackedDirectory(path: string): TFlow {
    const sessionId: Nullable<SessionId> = this.sessionId;

    if (!sessionId) {
      return;
    }

    yield* this.operation.run({
      name: "Export spawn file",
      subject: path,
      command: () => spawnCommands.saveUnpackedDirectory(path, sessionId).then(() => "export"),
      failure: "Could not export spawn file",
      success: () => ({ details: path, title: "Exported spawn file" }),
    });
  }

  /**
   * Dismisses the last reported write outcome.
   */
  @BoundAction()
  public clearOperation(): void {
    this.operation.reset();
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
   * Fetch one chunk, at most once; a cancelled read leaves it retryable.
   *
   * @param kind - The chunk to read.
   */
  private *fetchChunk<K extends ESpawnChunkKind>(kind: K): TFlow {
    const sessionId: Nullable<SessionId> = this.sessionId;
    const read: CommandOperation<TSpawnChunkOf<K>> = this.reads[kind] as CommandOperation<TSpawnChunkOf<K>>;

    if (!sessionId || read.isLoading || read.state.isReady) {
      return;
    }

    yield* read.run({
      name: `Read spawn ${kind} chunk`,
      subject: this.path,
      command: () => spawnCommands.readChunk(sessionId, kind).then((answer) => toChunkOf(answer, kind)),
      failure: `Could not read the ${kind} chunk`,
    });
  }

  /**
   * Lets every chunk read of the last file go, and the row selected in one of them.
   */
  @BoundAction()
  private forgetChunks(): void {
    // Abandoned rather than merely cleared: a read already on the wire would otherwise land under the next file.
    for (const lane of CHUNK_LANES) {
      cancelFlow(this, lane);
    }

    for (const read of Object.values(this.reads) as Array<CommandOperation<SpawnChunk>>) {
      read.reset();
    }

    // A selection outlives its table, so it has to be dropped with the data it pointed into.
    this.selectedRow = null;
  }
}

/**
 * @param answer - What `read_chunk` answered.
 * @param kind - What it was asked for.
 * @returns The chunk, of the kind asked for.
 * @throws {Error} When the backend answered another kind.
 */
function toChunkOf<K extends ESpawnChunkKind>(answer: SessionSnapshot<SpawnChunk>, kind: K): TSpawnChunkOf<K> {
  if (!isSpawnChunkOf(answer.value, kind)) {
    throw new Error(`Asked for the ${kind} chunk, read the ${answer.value.kind} chunk`);
  }

  return answer.value;
}
