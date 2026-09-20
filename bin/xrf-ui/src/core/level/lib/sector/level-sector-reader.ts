import { transformError } from "@/core/error/lib";
import { levelsCommands } from "@/core/ipc/commands/levels";
import { levelsRawCommands } from "@/core/ipc/commands/levels-raw";
import { SessionSnapshot } from "@/core/ipc/types/xrf-app";
import { XraySurfaceDescriptor } from "@/core/ipc/types/xrf-material";
import { SectorDescription } from "@/core/ipc/types/xrf-visual";
import { ISectorTextureRequest, listSectorTextures } from "@/core/level/lib/sector/level-sector-textures";
import { createSectorViews, ISectorViews } from "@/core/level/lib/sector/level-sector-views";
import { formatDuration } from "@/lib/format/duration";
import { Logger, Timer } from "@/lib/logging";
import { Maybe } from "@/lib/types/general";

/** What one sector read needs of whoever owns the level, so the reader owns none of it. */
export interface ILevelSectorReaderHost {
  /** The level's uploaded textures, which the reader adds to and never disposes. */
  load(requests: ReadonlyArray<ISectorTextureRequest>): Promise<void>;
  /** Whether that level opening is still the one held, checked once the read has everything in hand. */
  isOpen(sessionId: string): boolean;
  /** Takes a sector that arrived for a level still open. */
  adopt(views: ISectorViews): void;
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
   * @param surfaces - The level's resolved shader table, for the sector to join its surfaces against.
   * @returns A promise that settles when the sector has been read, or when it has failed and said so.
   */
  public read(sessionId: string, sector: number, surfaces: ReadonlyArray<XraySurfaceDescriptor>): Promise<void> {
    const key: string = `${sessionId}:${sector}`;
    const reading: Maybe<Promise<void>> = this.reading.get(key);

    if (reading) {
      return reading;
    }

    const started: Promise<void> = this.readSector(key, sessionId, sector, surfaces)
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

  private async readSector(
    key: string,
    sessionId: string,
    sector: number,
    surfaces: ReadonlyArray<XraySurfaceDescriptor>
  ): Promise<void> {
    const timer: Timer = new Timer();

    const snapshot: SessionSnapshot<SectorDescription> = await levelsCommands.openSector(
      sessionId,
      crypto.randomUUID(),
      sector
    );

    const buffer: ArrayBuffer = await levelsRawCommands.readSector(sessionId, snapshot.sessionId);
    // Joined against the table the open resolved, so a surface arrives already knowing whether it is cut out.
    const views: ISectorViews = createSectorViews(snapshot.value, buffer, surfaces);

    // Claimed before the upload rather than after it: what protects these textures has to be in place before
    // anything can be disposed for not being resident.
    const requested: ReadonlyArray<ISectorTextureRequest> = listSectorTextures(views);

    this.claimed.set(key, requested);

    // Before the sector is published, so a surface is never drawn untextured for a frame and then corrected.
    await this.host.load(requested);

    // A read outlives the plan that asked for it, so it can outlive the level too: a sector of a level nobody has
    // open any more is dropped rather than adopted into whatever is open now.
    if (!this.host.isOpen(sessionId)) {
      this.log.info(`Dropping sector ${sector} of a level that is no longer open`);

      return;
    }

    this.host.adopt(views);

    this.log.info(
      "Sector read in:",
      formatDuration(timer.elapsed()),
      `sector ${sector},`,
      `${views.geometry.vertexCount} vertices,`,
      `${views.sections.length} draws,`,
      `${views.instances.length} instanced meshes`
    );
  }
}
