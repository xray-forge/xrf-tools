import { transformError } from "@/core/error/lib";
import { levelsCommands } from "@/core/ipc/commands/levels";
import { levelsRawCommands } from "@/core/ipc/commands/levels-raw";
import { SessionSnapshot } from "@/core/ipc/types/xrf-app";
import { SectorDescription } from "@/core/ipc/types/xrf-visual";
import { ILevelSectorDelivery } from "@/core/level/lib/render/level-render-protocol";
import { ISectorTextureRequest } from "@/core/level/lib/sector/level-sector-textures";
import { ILevelStreamReading } from "@/core/level/lib/stream/level-stream-profile";
import { Logger, Timer } from "@/lib/logging";
import { Maybe } from "@/lib/types/general";

/** What one sector read needs of whoever owns the level, so the reader owns none of it. */
export interface ILevelSectorReaderHost {
  /** The level's uploaded textures, which the reader adds to and never disposes. */
  load(requests: ReadonlyArray<ISectorTextureRequest>): Promise<void>;
  /** What one sector's surfaces name, joined against the level's shader table. */
  listTextures(description: SectorDescription): ReadonlyArray<ISectorTextureRequest>;
  /** Whether that level opening is still the one held, checked once the read has everything in hand. */
  isOpen(sessionId: string): boolean;
  /** Takes a sector that arrived for a level still open, as the pack and the bytes it was packed into. */
  deliver(delivery: ILevelSectorDelivery): void;
  /** Takes what that sector cost, stage by stage. */
  record(reading: ILevelStreamReading): void;
}

/**
 * Reads sectors, one read per sector at a time, and says what is still in flight.
 */
export class LevelSectorReader {
  public readonly log: Logger = new Logger(__MODULE_NAME__);

  private readonly host: ILevelSectorReaderHost;

  /** The read in flight for each sector, so a second ask joins it rather than starting another. */
  private readonly reading: Map<string, Promise<void>> = new Map();

  /** What each read in flight has claimed, so nothing evicts a texture a sector on its way is about to draw with. */
  private readonly claimed: Map<string, ReadonlyArray<ISectorTextureRequest>> = new Map();

  public constructor(host: ILevelSectorReaderHost) {
    this.host = host;
  }

  /**
   * Reads one sector, or joins the read already in flight for it.
   *
   * @param sessionId - The level opening the sector belongs to.
   * @param sector - Sector to read, by its index in the sectors chunk.
   * @returns A promise that settles when the sector has been read, or when it has failed and said so.
   */
  public read(sessionId: string, sector: number): Promise<void> {
    const key: string = `${sessionId}:${sector}`;
    const reading: Maybe<Promise<void>> = this.reading.get(key);

    if (reading) {
      return reading;
    }

    const started: Promise<void> = this.readSector(key, sessionId, sector)
      .catch((error: unknown) => {
        this.log.error(`Failed to read sector ${sector}:`, transformError(error));
      })
      .finally(() => {
        this.reading.delete(key);
        // Released only once the sector is resident or has been dropped, which is what closes the window above.
        this.claimed.delete(key);
      });

    this.reading.set(key, started);

    return started;
  }

  /**
   * @returns Every texture reference a read still in flight has claimed, which is not resident and must not be
   *   disposed.
   */
  public listClaimedTextures(): Array<string> {
    return Array.from(this.claimed.values()).flatMap((requests) => requests.map((it) => it.reference));
  }

  private async readSector(key: string, sessionId: string, sector: number): Promise<void> {
    const timer: Timer = new Timer();
    const stage: Timer = new Timer();

    const snapshot: SessionSnapshot<SectorDescription> = await levelsCommands.openSector(
      sessionId,
      crypto.randomUUID(),
      sector
    );

    const pack: number = stage.lap();
    const buffer: ArrayBuffer = await levelsRawCommands.readSector(sessionId, snapshot.sessionId);
    const transfer: number = stage.lap();
    // Read from the description rather than from views over the bytes: what a sector's surfaces name is the
    // shader table's answer, and this side of the boundary never looks inside the pack.
    const requested: ReadonlyArray<ISectorTextureRequest> = this.host.listTextures(snapshot.value);
    const built: number = stage.lap();

    this.claimed.set(key, requested);

    // Before the sector is published, so a surface is never drawn untextured for a frame and then corrected.
    await this.host.load(requested);

    const textures: number = stage.lap();

    // A read outlives the plan that asked for it, so it can outlive the level too: a sector of a level nobody has
    // open any more is dropped rather than adopted into whatever is open now.
    if (!this.host.isOpen(sessionId)) {
      this.log.info(`Dropping sector ${sector} of a level that is no longer open`);

      return;
    }

    const stage2: Timer = new Timer();

    this.host.deliver({ buffer, description: snapshot.value, sector });

    this.host.record({
      deliver: stage2.elapsed(),
      draws: snapshot.value.sections.length + snapshot.value.instances.length,
      pack,
      sector,
      textures,
      total: timer.elapsed(),
      transfer,
      vertices: snapshot.value.geometry.vertexCount,
      views: built,
    });
  }
}
