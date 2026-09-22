import { Injectable, OnDeactivation, OnProvision } from "@wirestate/core";
import { BoundAction, Computed, flowResult, Observable, runInAction } from "@wirestate/mobx";

import { transformError } from "@/core/error/lib";
import { levelsCommands } from "@/core/ipc/commands/levels";
import { Session } from "@/core/ipc/session";
import { requireSessionId } from "@/core/ipc/session/session.utils";
import { LevelSource, SelectedLevelDescription, SessionSnapshot } from "@/core/ipc/types/xrf-app";
import { XrayRoots } from "@/core/ipc/types/xrf-vfs";
import { SectorDescription } from "@/core/ipc/types/xrf-visual";
import {
  ILevelSectorChange,
  ILevelSectorDelivery,
  ILevelSectorSource,
  ILevelTextureDelivery,
  ILevelTextureSupply,
  ILevelTextureSupplyChange,
  TLevelSectorListener,
  TLevelTextureSupplyListener,
} from "@/core/level/lib/render/level-render-protocol";
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
import { Maybe, Nullable } from "@/lib/types/general";

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
  private readonly supplied: Map<string, ISectorTextureRequest> = new Map();

  /** What the reads have cost, kept here because the loader is what owns a read from end to end. */
  private readonly profile: LevelStreamProfile = new LevelStreamProfile();

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
    load: (requests: ReadonlyArray<ISectorTextureRequest>): Promise<void> => this.supply(requests),
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
   * Reads the files a sector's surfaces name, and supplies them to whatever uploads them.
   *
   * @param requests - What the sector names, base textures and lightmaps alike.
   */
  private async supply(requests: ReadonlyArray<ISectorTextureRequest>): Promise<void> {
    const wanted: Array<ISectorTextureRequest> = requests.filter((request: ISectorTextureRequest) => {
      const supplied: Maybe<ISectorTextureRequest> = this.supplied.get(request.reference);

      return (
        Boolean(request.reference) &&
        (!supplied || (request.isAlphaRead && !supplied.isAlphaRead) || (!request.isMipped && supplied.isMipped))
      );
    });

    if (!wanted.length) {
      return;
    }

    for (const request of wanted) {
      this.supplied.set(request.reference, request);
    }

    const delivered: Array<ILevelTextureDelivery> = await Promise.all(
      wanted.map((request: ISectorTextureRequest) => this.reading.read(request))
    );

    this.notifyTextures({ delivered, retained: null });
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

    this.log.info("Loading level:", source.kind === "directory" ? source.path : source.logicalPath);

    try {
      this.level = this.level.asLoading();

      const selected: SessionSnapshot<SelectedLevelDescription> = yield* call(
        this.session.open(levelsCommands.openLevel, source, roots)
      );

      this.adopt(selected);

      this.log.info(
        "Level opened in:",
        formatDuration(timer.elapsed()),
        source.kind === "directory" ? source.path : source.logicalPath,
        `${selected.value.sectors.length} sectors,`,
        `${selected.value.visuals} visuals`
      );
    } catch (error: unknown) {
      const transformed: Error = transformError(error);

      this.log.error(
        "Failed to load level:",
        source.kind === "directory" ? source.path : source.logicalPath,
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
    const snapshot = yield* call(levelsCommands.getLevel());

    this.session.adopt(snapshot);

    if (snapshot) {
      this.adopt(snapshot);

      const source: LevelSource = snapshot.value.source;

      this.log.info(
        "Level restored:",
        source.kind === "directory" ? source.path : source.logicalPath,
        snapshot.sessionId
      );
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
    // in it. The level about to open inherits neither.
    this.scheduler.clear();

    this.residency = createLevelResidency(selected.value.bounds?.boundingSphere.radius ?? 0);
    this.scheduler.setConcurrency(this.residency.concurrency);
    // The roots the **open** searched, not the ones a caller happens to hold: they are centred on the level, which is
    // what finds a texture shipped beside it, and a restore has no other way to know them. Releases the last level's
    // textures itself, and says so once rather than once for the release and again for the open.
    this.supplied.clear();
    this.reading.open(selected.value.roots, selected.value.textures);
    this.notifyTextures({ delivered: [], retained: null });
    this.level = this.level.asReady({ selected });
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

    for (const reference of Array.from(this.supplied.keys())) {
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
    const source = this.level.value?.selected.value.source;

    yield* call(this.session.close());

    this.clearView();

    if (source) {
      this.log.info("Level closed:", source.kind === "directory" ? source.path : source.logicalPath);
    }
  }

  /**
   * Forgets what is resident and reads it again, for a level that has to be handed to a different renderer.
   */
  @BoundAction()
  public restream(): Promise<void> {
    const from: Nullable<ILevelPoint> = this.streamedFrom;

    this.scheduler.clear();
    this.supplied.clear();
    this.releaseSectors();

    this.streamedFrom = null;

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

    for (const reference of this.reader.listClaimedTextures()) {
      references.add(reference);
    }

    return references;
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
    this.scheduler.clear();
    this.profile.clear();
    this.streamedFrom = null;
    this.streamProfile = EMPTY_LEVEL_STREAM_SUMMARY;

    runInAction(() => {
      this.level = this.level.asIdle();
      this.streaming = IDLE_LEVEL_STREAM;
      this.releaseSectors();
      this.supplied.clear();
      this.reading.close();
      this.notifyTextures({ delivered: [], retained: null });
    });
  }

  private releaseSectors(): void {
    this.ledger.clear();
    this.sectorReport = EMPTY_LEVEL_SECTOR_REPORT;
    this.notifySectors({ delivered: [], released: null });
  }
}
