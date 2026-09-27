import { Injectable, OnDeactivation, OnProvision } from "@wirestate/core";
import { BoundAction, Computed, flowResult, Observable, runInAction } from "@wirestate/mobx";
import { Maybe, Nullable } from "@xrf/types";

import { transformError } from "@/core/error/lib";
import { levelsCommands } from "@/core/ipc/commands/levels";
import { levelsRawCommands } from "@/core/ipc/commands/levels-raw";
import { Session } from "@/core/ipc/session";
import { requireSessionId } from "@/core/ipc/session/session.utils";
import {
  LevelDetailsDescription,
  LevelLightsDescription,
  LevelSource,
  LevelSpawnModelsDescription,
  LevelTextureReference,
  SelectedLevelDescription,
  SessionRestore,
  SessionSnapshot,
} from "@/core/ipc/types/xrf-app";
import { XrayRoots } from "@/core/ipc/types/xrf-vfs";
import { SectorDescription } from "@/core/ipc/types/xrf-visual";
import { LevelHeld } from "@/core/level/lib/render/level-held";
import {
  ILevelGrassDelivery,
  ILevelHeldSource,
  ILevelSectorChange,
  ILevelSectorDelivery,
  ILevelSectorSource,
  ILevelTextureDelivery,
  ILevelTextureSupply,
  ILevelTextureSupplyChange,
  TLevelSectorListener,
  TLevelTextureSupplyListener,
} from "@/core/level/lib/render/level-render-protocol";
import { ILevelSpawnModelsDelivery } from "@/core/level/lib/render/level-render-spawn";
import {
  createLevelResidency,
  DEFAULT_LEVEL_RESIDENCY,
  ILevelPoint,
  ILevelResidencyOptions,
  ILevelResidencyPlan,
  listLevelPreload,
  planLevelResidency,
} from "@/core/level/lib/residency/level-residency";
import { LevelSectorReader } from "@/core/level/lib/sector/level-sector-reader";
import {
  EMPTY_LEVEL_SECTOR_REPORT,
  ILevelSectorReport,
  ILevelSectorSkip,
} from "@/core/level/lib/sector/level-sector-report";
import { ISectorTextureRequest, listDescriptionTextures } from "@/core/level/lib/sector/level-sector-textures";
import { describeLevelSource } from "@/core/level/lib/source";
import {
  EMPTY_LEVEL_STREAM_SUMMARY,
  ILevelStreamReading,
  ILevelStreamSummary,
  LevelStreamProfile,
} from "@/core/level/lib/stream/level-stream-profile";
import { LevelStreamScheduler } from "@/core/level/lib/stream/level-stream-scheduler";
import { LevelTextureReader } from "@/core/level/lib/texture/level-texture-reader";
import { AsyncState } from "@/lib/async-state";
import { formatDuration } from "@/lib/format/duration";
import { Logger, Timer } from "@/lib/logging";
import { call, cancelFlow, ExclusiveFlow, LatestFlow, TFlow } from "@/lib/mobx";

/** Nothing in flight, which is also what a viewer sees before it has asked for anything. */
export const IDLE_LEVEL_STREAM: ILevelStreamProgress = { loaded: 0, total: 0 };

/** A level that is open: what it is, and where its sectors are. */
export interface IOpenLevel {
  selected: SessionSnapshot<SelectedLevelDescription>;
}

/** What the loader keeps of a sector it has handed on, which is everything it needs to plan against. */
interface ILevelSectorEntry {
  /** Bytes of the pack, which is what the memory budget is spent on. */
  bytes: number;
  /** What its surfaces name, so retention can be decided without looking inside the pack. */
  textures: ReadonlyArray<ISectorTextureRequest>;
  /** What the pack could not read. */
  skipped: ReadonlyArray<ILevelSectorSkip>;
}

/** What one read of something a level holds came to: the value, what it binds, and a line saying how much it was. */
interface ILevelHeldRead<T> {
  value: T;
  textures: ReadonlyArray<LevelTextureReference>;
  summary: ReadonlyArray<string>;
}

/** How far through the sectors a camera asked for the loader has got. */
export interface ILevelStreamProgress {
  /** Sectors the current plan asked for, or zero when nothing is streaming. */
  total: number;
  /** Sectors of it that have arrived. */
  loaded: number;
}

/**
 * Opens a compiled level and keeps the sectors near the camera resident.
 */
@Injectable()
export class LevelLoadService {
  public readonly log: Logger = new Logger(__MODULE_NAME__);

  private readonly session: Session = new Session(levelsCommands.closeLevel);

  /** What has been handed on, and what each of them cost. */
  private readonly ledger: Map<number, ILevelSectorEntry> = new Map();

  /** Told what has been delivered and what has gone, which is how whatever draws the level hears of it. */
  private readonly watchers: Set<TLevelSectorListener> = new Set();

  /** Reads the files a level's textures come from, which is an `invoke` and so belongs on this side. */
  private readonly reading: LevelTextureReader = new LevelTextureReader();

  /** Told what files have been read and what is still worth keeping. */
  private readonly textureWatchers: Set<TLevelTextureSupplyListener> = new Set();

  /** References already supplied, so a second sector naming one does not read the file again. */
  private readonly supplied: Set<string> = new Set();

  /** What the reads have cost, kept here because the loader is what owns a read from end to end. */
  private readonly profile: LevelStreamProfile = new LevelStreamProfile();

  /**
   * What the level holds besides its sectors, each read once after it opens and handed to a renderer started later
   * too: its grass, its lights and the projectors they sample, and the models its spawned objects stand as.
   */
  private readonly heldGrass: LevelHeld<ILevelGrassDelivery> = new LevelHeld();
  private readonly heldLights: LevelHeld<LevelLightsDescription> = new LevelHeld();
  private readonly heldSpawnModels: LevelHeld<ILevelSpawnModelsDelivery> = new LevelHeld();
  /** The sky cube the level is lit under, which its water reflects. */
  private readonly heldSky: LevelHeld<LevelTextureReference> = new LevelHeld();
  /** The open level's grass, lights and spawned models being read, settling once all three have. */
  private heldReads: Promise<void> = Promise.resolve();

  /** Where the camera last reported from, which is what the level is filled in around once it settles. */
  private streamedFrom: Nullable<ILevelPoint> = null;

  /**
   * Reads sectors on this loader's behalf, joining a read already in flight and holding its textures until the
   * sector it read is resident or dropped.
   */
  private readonly reader: LevelSectorReader = new LevelSectorReader({
    deliver: (delivery: ILevelSectorDelivery): void => this.takeDelivery(delivery),
    isOpen: (sessionId: string): boolean => this.isOpen(sessionId),
    listTextures: (description: SectorDescription): ReadonlyArray<ISectorTextureRequest> =>
      this.listTextures(description),
    load: (sessionId: string, requests: ReadonlyArray<ISectorTextureRequest>): Promise<number> =>
      this.supply(sessionId, requests),
    record: (reading: ILevelStreamReading): void => this.noteReading(reading),
  });

  /**
   * Brings what the camera wants into residency.
   *
   * A camera report sets its target and returns; it starts nothing itself and cancels nothing, which is what keeps
   * a flight from spending every frame abandoning the read it asked for on the frame before.
   */
  private readonly scheduler: LevelStreamScheduler = new LevelStreamScheduler({
    isResident: (sector: number): boolean => this.ledger.has(sector),
    read: (sector: number): Promise<void> => this.readSector(sector),
    report: (loaded: number, total: number): void => this.noteProgress(loaded, total),
    settle: (): void => this.onSettled(),
  });

  @Observable()
  public level: AsyncState<IOpenLevel> = AsyncState.idle();

  /** What is held, for a viewer to report: how many, how much, and what their packs could not read. */
  @Observable()
  public sectorReport: ILevelSectorReport = EMPTY_LEVEL_SECTOR_REPORT;

  /** How much of the level is held at once, and how far out it is worth holding. */
  @Observable()
  public residency: ILevelResidencyOptions = DEFAULT_LEVEL_RESIDENCY;

  @Observable()
  public streaming: ILevelStreamProgress = IDLE_LEVEL_STREAM;

  /** What the recent sector reads cost, stage by stage, for a viewer to report and a change to be judged against. */
  @Observable()
  public streamProfile: ILevelStreamSummary = EMPTY_LEVEL_STREAM_SUMMARY;

  /**
   * Whether the restore below has settled, one way or the other.
   */
  @Observable()
  public isReady: boolean = false;

  /** The level's grass, told as it is now and whenever it changes. */
  public get grass(): ILevelHeldSource<ILevelGrassDelivery> {
    return this.heldGrass;
  }

  /** The level's lights, told as they are now and whenever they change. */
  public get lights(): ILevelHeldSource<LevelLightsDescription> {
    return this.heldLights;
  }

  /** The models the level's spawned objects stand as, told as they are now and whenever they change. */
  public get spawnModels(): ILevelHeldSource<ILevelSpawnModelsDelivery> {
    return this.heldSpawnModels;
  }

  /**
   * @returns The level's textures, to read and to hear about rather than to manage: their lifetime is this
   *   service's, and the set says for itself what changed.
   */
  public get textures(): ILevelTextureSupply {
    return {
      subscribe: (listener: TLevelTextureSupplyListener): (() => void) => {
        this.textureWatchers.add(listener);

        return (): void => {
          this.textureWatchers.delete(listener);
        };
      },
    };
  }

  /**
   * @returns The level's sectors, to draw rather than to read: a description and its bytes, never a geometry.
   */
  public get sectors(): ILevelSectorSource {
    return {
      subscribe: (listener: TLevelSectorListener): (() => void) => {
        this.watchers.add(listener);

        return (): void => {
          this.watchers.delete(listener);
        };
      },
    };
  }

  /**
   * @returns Whether sectors are on their way.
   */
  @Computed()
  public get isStreaming(): boolean {
    return this.streaming.total > 0;
  }

  @OnDeactivation()
  public onDeactivation(): void {
    this.clear();
  }

  /**
   * Reads the files a level's surfaces name, and supplies them to whatever uploads them, for a level still open: a
   * read that outlives its level delivers nothing into the next.
   *
   * @param sessionId - The level opening that named them.
   * @param requests - What its sectors, grass, lights or spawned models name, base textures and lightmaps alike.
   * @returns How many files it read.
   */
  private async supply(sessionId: string, requests: ReadonlyArray<ISectorTextureRequest>): Promise<number> {
    const wanted: Array<ISectorTextureRequest> = requests.filter(
      (request: ISectorTextureRequest) => Boolean(request.reference) && !this.supplied.has(request.reference)
    );

    if (!wanted.length || !this.isOpen(sessionId)) {
      return 0;
    }

    for (const request of wanted) {
      this.supplied.add(request.reference);
    }

    const delivered: Array<ILevelTextureDelivery> = await Promise.all(
      wanted.map((request: ISectorTextureRequest) => this.reading.read(request))
    );

    if (!this.isOpen(sessionId)) {
      return 0;
    }

    this.notifyTextures({ delivered, retained: null });

    return wanted.length;
  }

  private notifyTextures(change: ILevelTextureSupplyChange): void {
    for (const watcher of Array.from(this.textureWatchers)) {
      watcher(change);
    }
  }

  /**
   * Open a level and report what it is built out of.
   *
   * @param source - Level directory or mounted asset to open.
   * @param roots - Roots the level and its textures are searched in.
   */
  @LatestFlow("level")
  public *load(source: LevelSource, roots: XrayRoots): TFlow {
    const timer: Timer = new Timer();

    this.log.info("Loading level:", describeLevelSource(source));

    try {
      this.level = this.level.asLoading();

      const selected: SessionSnapshot<SelectedLevelDescription> = yield* call(
        this.session.open(levelsCommands.openLevel, source, roots)
      );

      this.adopt(selected);

      this.log.info(
        "Level opened in:",
        formatDuration(timer.elapsed()),
        describeLevelSource(source),
        `${selected.value.sectors.length} sectors,`,
        `${selected.value.visuals} visuals`
      );
    } catch (error: unknown) {
      const transformed: Error = transformError(error);

      this.log.error(
        "Failed to load level:",
        describeLevelSource(source),
        "after",
        formatDuration(timer.elapsed()),
        transformed
      );

      this.level = this.level.asFailed(transformed);
    }
  }

  /**
   * Takes back whatever level the backend still has open.
   */
  @OnProvision()
  public async onProvision(): Promise<void> {
    try {
      await flowResult(this.restore());
    } catch (error: unknown) {
      this.log.error("Failed to restore the selected level:", transformError(error));
    } finally {
      runInAction(() => {
        this.isReady = true;
      });
    }
  }

  /**
   * Restores the native selection unless a user action has taken the level flow.
   */
  @ExclusiveFlow("level")
  public *restore(): TFlow {
    const snapshot: SessionRestore<SelectedLevelDescription> = yield* call(levelsCommands.getLevel());

    this.session.adopt(snapshot);

    if (snapshot) {
      this.adopt(snapshot);
      this.log.info("Level restored:", describeLevelSource(snapshot.value.source), snapshot.sessionId);
    }
  }

  /**
   * Takes one opening as the level this loader holds.
   *
   * @param selected - The opening the backend holds, from an open or from a restore.
   */
  @BoundAction()
  private adopt(selected: SessionSnapshot<SelectedLevelDescription>): void {
    this.releaseSectors();

    // Before anything else: a target belongs to the level it was planned against, and so does every sector number
    // in it. The level about to open inherits neither, nor what streaming the last one cost.
    this.resetStream();

    this.residency = createLevelResidency(selected.value.bounds?.boundingSphere.radius ?? 0);
    this.scheduler.setConcurrency(this.residency.concurrency);
    // The roots the **open** searched, not the ones a caller happens to hold: they are centred on the level, which is
    // what finds a texture shipped beside it, and a restore has no other way to know them. Releases the last level's
    // textures itself, and says so once rather than once for the release and again for the open.
    this.supplied.clear();
    this.reading.open(selected.value.roots, selected.value.textures);
    this.notifyTextures({ delivered: [], retained: null });
    this.level = this.level.asReady({ selected });
    this.releaseHeld();
    this.heldReads = Promise.all([
      this.readHeld(selected.sessionId, "grass", this.heldGrass, () => this.readGrass(selected.sessionId)),
      this.readHeld(selected.sessionId, "lights", this.heldLights, () => this.readLights(selected.sessionId)),
      this.readHeld(selected.sessionId, "spawned models", this.heldSpawnModels, () =>
        this.readSpawnModels(selected.sessionId)
      ),
      this.readHeld(selected.sessionId, "sky", this.heldSky, () =>
        Promise.resolve({
          summary: [selected.value.sky.reference],
          textures: [selected.value.sky],
          value: selected.value.sky,
        })
      ),
    ]).then(() => undefined);
  }

  /**
   * @returns Settles once the open level's grass, lights and spawned models are held, or have failed and said so;
   *   each is handed on as it is held, so what draws them has them all by then.
   */
  public whenHeldRead(): Promise<void> {
    return this.heldReads;
  }

  /**
   * Reads one thing the level holds, supplies the textures it binds, then holds it, for a level still open after each
   * step: its textures claimed before they are read, so a settle meanwhile keeps them.
   *
   * @param sessionId - The level opening it belongs to.
   * @param what - What it is, as the log names it.
   * @param held - Where it is held.
   * @param read - Reads it, or answers null for a level holding none.
   */
  private async readHeld<T>(
    sessionId: string,
    what: string,
    held: LevelHeld<T>,
    read: () => Promise<Nullable<ILevelHeldRead<T>>>
  ): Promise<void> {
    const timer: Timer = new Timer();

    try {
      const result: Nullable<ILevelHeldRead<T>> = await read();

      if (!result || !this.isOpen(sessionId)) {
        return;
      }

      this.reading.add(result.textures);
      held.claim(new Set(result.textures.map((it: LevelTextureReference) => it.reference)));
      await this.supply(
        sessionId,
        result.textures.map((it: LevelTextureReference) => ({ reference: it.reference }))
      );

      if (!this.isOpen(sessionId)) {
        return;
      }

      this.log.info(`Level ${what} read in:`, formatDuration(timer.elapsed()), ...result.summary);
      held.hold(result.value);
    } catch (error: unknown) {
      this.log.error(`Failed to read the level's ${what}:`, transformError(error));
    }
  }

  /** The level's lights, and the projectors its spots sample. */
  private async readLights(sessionId: string): Promise<ILevelHeldRead<LevelLightsDescription>> {
    const { value }: SessionSnapshot<LevelLightsDescription> = await levelsCommands.openLights(sessionId);

    return {
      summary: [`${value.lights.lights.length} lights,`, `${value.lights.animators.length} animators`],
      textures: value.projectors,
      value,
    };
  }

  /** The models the level's spawned objects stand as, each one's pack read at once, and their textures. */
  private async readSpawnModels(sessionId: string): Promise<Nullable<ILevelHeldRead<ILevelSpawnModelsDelivery>>> {
    const { value: description }: SessionSnapshot<LevelSpawnModelsDescription> =
      await levelsCommands.openSpawnModels(sessionId);

    if (!description.models.length || !this.isOpen(sessionId)) {
      return null;
    }

    const packs: Array<ArrayBuffer> = await Promise.all(
      description.models.map((model) => levelsRawCommands.readSpawnModel(sessionId, model.name))
    );

    return {
      summary: [`${description.models.length} models,`, `${description.placements.length} placed`],
      textures: description.models.flatMap((model) => model.textures),
      value: {
        buffers: new Map(description.models.map((model, index: number) => [model.name, packs[index]])),
        description,
      },
    };
  }

  /** The level's grass, packed for it, and its textures; null for a level with none. */
  private async readGrass(sessionId: string): Promise<Nullable<ILevelHeldRead<ILevelGrassDelivery>>> {
    const snapshot: SessionSnapshot<Nullable<LevelDetailsDescription>> = await levelsCommands.openDetails(
      sessionId,
      crypto.randomUUID()
    );
    const description: Nullable<LevelDetailsDescription> = snapshot.value;

    if (!description || !this.isOpen(sessionId)) {
      return null;
    }

    return {
      summary: [`${description.details.slotCount} planted slots,`, `${description.details.models.length} models`],
      textures: description.textures,
      value: { buffer: await levelsRawCommands.readDetails(sessionId, snapshot.sessionId), description },
    };
  }

  /** Lets go of everything the level holds besides its sectors. */
  private releaseHeld(): void {
    this.heldGrass.release();
    this.heldLights.release();
    this.heldSpawnModels.release();
    this.heldSky.release();
  }

  /**
   * Brings what the camera is near into residency, and releases what it has left.
   *
   * @param point - Where the camera is, in renderer space.
   */
  public stream(point: ILevelPoint): Promise<void> {
    const open: Nullable<IOpenLevel> = this.level.value;

    if (!open) {
      return Promise.resolve();
    }

    const timer: Timer = new Timer();

    this.streamedFrom = point;

    const plan: ILevelResidencyPlan = planLevelResidency(
      open.selected.value.sectors,
      point,
      this.listSizes(),
      this.residency
    );

    if (plan.evict.length) {
      for (const sector of plan.evict) {
        this.ledger.delete(sector);
      }

      this.notifySectors({ delivered: [], released: plan.evict });
      this.publishSectorReport();
    }

    // The whole target, not the part of it that is missing: what is already held is what tells the scheduler which
    // of its reads are still worth finishing.
    const settled: Promise<void> = this.scheduler.setTarget(plan.resident);

    // Measured here rather than around the plan alone, because what a camera report costs the frame it lands on is
    // everything this method does - and answering one used to include abandoning whatever was in flight.
    this.noteReport(timer.elapsed());

    return settled;
  }

  /**
   * Reads one sector of the level this loader holds.
   *
   * @param sector - Sector to read, by its index in the sectors chunk.
   * @returns Settles when it is resident or has failed and said so.
   */
  private readSector(sector: number): Promise<void> {
    const open: Nullable<IOpenLevel> = this.level.value;

    return open ? this.reader.read(requireSessionId(open.selected), sector) : Promise.resolve();
  }

  /**
   * Takes one sector that has been read and hands it on.
   *
   * @param delivery - The pack and the bytes it was packed into.
   */
  private takeDelivery(delivery: ILevelSectorDelivery): void {
    const description: SectorDescription = delivery.description;

    this.ledger.set(delivery.sector, {
      bytes: description.bufferLength,
      skipped: description.skipped.map((skip) => ({ sector: delivery.sector, skip })),
      textures: this.listTextures(description),
    });

    this.notifySectors({ delivered: [delivery], released: [] });
    this.publishSectorReport();
  }

  /**
   * @param description - What `open_sector` reported about a pack.
   * @returns What its surfaces name, joined against the level's shader table.
   */
  private listTextures(description: SectorDescription): ReadonlyArray<ISectorTextureRequest> {
    return listDescriptionTextures(description, this.level.value?.selected.value.surfaces ?? []);
  }

  /** What each held sector cost, which is what a memory budget is planned against. */
  private listSizes(): ReadonlyMap<number, number> {
    return new Map(Array.from(this.ledger, ([sector, entry]) => [sector, entry.bytes]));
  }

  private notifySectors(change: ILevelSectorChange): void {
    for (const watcher of Array.from(this.watchers)) {
      watcher(change);
    }
  }

  /**
   * Everything the camera wants is resident or has failed.
   *
   * Retention happens here and nowhere else. It reads every resident sector's surfaces, which is far too much to
   * pay on a camera report - and under boost a report lands every frame.
   */
  @BoundAction()
  private onSettled(): void {
    const retained: Set<string> = this.listResidentTextures();

    for (const reference of Array.from(this.supplied)) {
      if (!retained.has(reference)) {
        this.supplied.delete(reference);
      }
    }

    this.notifyTextures({ delivered: [], retained });

    this.streaming = IDLE_LEVEL_STREAM;

    // Only now, and only from here: it walks every sector of the level, which is far too much for a camera
    // report, and there is nothing to fill in with until the camera has what it asked for.
    if (this.residency.isPreloaded) {
      const open: Nullable<IOpenLevel> = this.level.value;

      this.scheduler.setBackground(
        open && this.streamedFrom ? listLevelPreload(open.selected.value.sectors, this.streamedFrom, this.ledger) : []
      );
    }
  }

  @BoundAction()
  private noteProgress(loaded: number, total: number): void {
    this.streaming = { loaded, total };
  }

  /**
   * Takes what one camera report cost and republishes the summary.
   *
   * @param elapsed - Milliseconds it took on the thread that draws.
   */
  @BoundAction()
  private noteReport(elapsed: number): void {
    this.profile.recordReport(elapsed);
    this.streamProfile = this.profile.summarise();
  }

  /**
   * Takes what one sector cost and republishes the summary.
   *
   * @param reading - What that sector's read came to, stage by stage.
   */
  @BoundAction()
  private noteReading(reading: ILevelStreamReading): void {
    this.profile.record(reading);
    this.streamProfile = this.profile.summarise();
  }

  /**
   * Closes this loader's level without clearing a newer one.
   */
  @LatestFlow("level")
  public *close(): TFlow {
    const source: Maybe<LevelSource> = this.level.value?.selected.value.source;

    yield* call(this.session.close());

    this.clearView();

    if (source) {
      this.log.info("Level closed:", describeLevelSource(source));
    }
  }

  /**
   * Forgets what is resident and reads it again, for a level that has to be handed to a different renderer: the
   * sectors from where the camera was, and the textures of what the level holds, which the renderer is handed again.
   */
  @BoundAction()
  public restream(): Promise<void> {
    const from: Nullable<ILevelPoint> = this.streamedFrom;
    const open: Nullable<IOpenLevel> = this.level.value;

    this.scheduler.clear();
    this.supplied.clear();
    this.releaseSectors();

    this.streamedFrom = null;

    if (open) {
      const held: Set<string> = this.listHeldTextures();

      void this.supply(
        open.selected.sessionId,
        Array.from(held, (reference: string) => ({ reference }))
      );
    }

    return from ? this.stream(from) : Promise.resolve();
  }

  /**
   * Abandons pending work and releases the level and every sector it held.
   */
  @BoundAction()
  public clear(): void {
    cancelFlow(this, "level");
    this.scheduler.clear();
    this.session.release();

    this.clearView();
  }

  /**
   * @param sessionId - The level opening a read belongs to.
   * @returns Whether that level is still the one this loader holds.
   */
  private isOpen(sessionId: string): boolean {
    const open: Nullable<IOpenLevel> = this.level.value;

    return Boolean(open && open.selected.sessionId === sessionId);
  }

  /**
   * @returns Every texture reference a resident sector names or a read in flight has claimed, which is what is worth
   *   keeping uploaded.
   */
  private listResidentTextures(): Set<string> {
    const references: Set<string> = new Set();

    for (const entry of this.ledger.values()) {
      for (const request of entry.textures) {
        references.add(request.reference);
      }
    }

    for (const reference of [...this.reader.listClaimedTextures(), ...this.listHeldTextures()]) {
      references.add(reference);
    }

    return references;
  }

  /** @returns What the level's grass, lights and spawned models bind, held or claimed. */
  private listHeldTextures(): Set<string> {
    return new Set([
      ...this.heldGrass.textures,
      ...this.heldLights.textures,
      ...this.heldSpawnModels.textures,
      ...this.heldSky.textures,
    ]);
  }

  private publishSectorReport(): void {
    let bytes: number = 0;

    const skipped: Array<ILevelSectorSkip> = [];

    for (const entry of this.ledger.values()) {
      bytes += entry.bytes;
      skipped.push(...entry.skipped);
    }

    runInAction(() => {
      this.sectorReport = { bytes, held: Array.from(this.ledger.keys()), skipped };
    });
  }

  private clearView(): void {
    runInAction(() => {
      this.resetStream();
      this.level = this.level.asIdle();
      this.releaseSectors();
      this.supplied.clear();
      this.reading.close();
      this.notifyTextures({ delivered: [], retained: null });
      this.releaseHeld();
    });
  }

  /** Forgets what streaming the held level asked for, from where, and what it cost. */
  private resetStream(): void {
    this.scheduler.clear();
    this.profile.clear();
    this.streamedFrom = null;
    this.streamProfile = EMPTY_LEVEL_STREAM_SUMMARY;
    this.streaming = IDLE_LEVEL_STREAM;
  }

  private releaseSectors(): void {
    this.ledger.clear();
    this.sectorReport = EMPTY_LEVEL_SECTOR_REPORT;
    this.notifySectors({ delivered: [], released: null });
  }
}
