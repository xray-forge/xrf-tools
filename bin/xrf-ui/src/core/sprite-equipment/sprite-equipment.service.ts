import { clamp } from "@mui/x-data-grid/internals";
import { path } from "@tauri-apps/api";
import { convertFileSrc } from "@tauri-apps/api/core";
import { exists } from "@tauri-apps/plugin-fs";
import { EventBus, inject, Injectable, OnDeactivation, OnProvision } from "@wirestate/core";
import { BoundAction, Computed, flowResult, Observable } from "@wirestate/mobx";

import { describePackSpriteOutcome } from "@/applications/sprite-equipment-packer/lib/describe-pack-sprite-outcome";
import { urlToImage } from "@/core/assets/image";
import { AssetService } from "@/core/assets/services";
import { spriteEquipmentCommands } from "@/core/bindings/commands/sprite-equipment";
import { transformError } from "@/core/error/lib";
import { releaseEditorProject } from "@/core/ipc/release";
import { EJobKind, IJobNotice, IJobOutcome, IJobRun, IJobState } from "@/core/jobs/lib";
import { JobsService } from "@/core/jobs/services/jobs";
import { emitNotification, ENotificationSeverity } from "@/core/notifications/lib";
import { EApplicationGroupId } from "@/core/routing/application";
import {
  IEquipmentSectionDescriptor,
  IEquipmentSpriteMetadata,
  IPackEquipmentResult,
} from "@/core/sprite-equipment/equipment";
import { createLoadable, Loadable } from "@/lib/loadable";
import { Logger } from "@/lib/logging";
import { all, call, ExclusiveFlow, LatestFlow, TFlow } from "@/lib/mobx";
import { Nullable } from "@/lib/types/general";

export interface IEquipmentPngDescriptor {
  ltxPath: string;
  /** Whether the open project's descriptors came out of a DLTX-resolved config tree. */
  isDltx: boolean;
  descriptors: Array<IEquipmentSectionDescriptor>;
  path: string;
  name: string;
  blob: Blob;
  image: HTMLImageElement;
}

/** One sprite is open at a time, so its url lives under a fixed key rather than being tracked by hand. */
const SPRITE_ASSET_KEY: string = "equipment-sprite";

@Injectable()
export class SpriteEquipmentService {
  public readonly log: Logger = new Logger(__MODULE_NAME__);

  @Observable()
  public isReady: boolean = false;

  @Observable()
  public isGridVisible: boolean = true;

  @Observable()
  public gridSize: number = 50;

  @Observable()
  public spriteImage: Loadable<Nullable<IEquipmentPngDescriptor>> = createLoadable(null);

  /**
   * Directory the sprite can be rebuilt from, or null when there is nothing to rebuild from.
   */
  @Observable()
  public repackSourcePath: Nullable<string> = null;

  /** Timestamp of the last successful repack, so the status bar can confirm the write happened. */
  @Observable()
  public repackedAt: Nullable<number> = null;

  /** The sprite pack this service started, while it runs. */
  @Observable()
  public packJobId: Nullable<string> = null;

  public constructor(
    private readonly assetService: AssetService = inject(AssetService),
    private readonly eventBus: EventBus = inject(EventBus),
    private readonly jobsService: JobsService = inject(JobsService)
  ) {}

  @OnProvision()
  public async onProvision(): Promise<void> {
    await flowResult(this.restore());
  }

  /**
   * Release the sprite when the editor is navigated away from.
   */
  @OnDeactivation()
  public onDeactivation(): void {
    this.assetService.releaseKey(SPRITE_ASSET_KEY);
    releaseEditorProject(spriteEquipmentCommands.closeSprite);
  }

  /**
   * Puts back whatever the backend already had open.
   *
   * Exclusive rather than latest. A restore must lose to anything the user started: joining the lane
   * leaves an open in progress alone, where superseding would cancel the very thing the user asked for. The user's
   * own actions take the lane the other way round, so an open cancels a restore that is still in flight.
   */
  @ExclusiveFlow("spriteImage")
  private *restore(): TFlow {
    const response: Nullable<IEquipmentSpriteMetadata> = yield* call(spriteEquipmentCommands.getSprite());

    if (!response) {
      this.log.info("No existing sprite detected file");
      this.isReady = true;

      return;
    }

    this.log.info("Existing equipment sprite detected");
    this.isReady = true;

    const spriteImage: IEquipmentPngDescriptor = yield* call(this.spriteFromResponse(response));

    this.spriteImage = createLoadable(spriteImage);

    yield* this.resolveRepackSource(spriteImage.path);
  }

  @BoundAction()
  public setGridVisibility(isVisible: boolean): void {
    this.isGridVisible = isVisible;
  }

  /**
   * Dismisses a reported failure while keeping the displayed sprite.
   */
  @BoundAction()
  public clearSpriteError(): void {
    this.spriteImage = this.spriteImage.asReady();
  }

  @BoundAction()
  public setGridSize(size: number): void {
    this.gridSize = Math.round(clamp(size, 10, 100));
  }

  /**
   * Reads a sprite sheet and the configuration naming its icons.
   *
   * @param equipmentDdsPath - The packed `*.dds` holding the inventory icons.
   * @param systemLtxPath - `system.ltx` declaring which icons exist and where they sit.
   * @param isDltx - Whether to resolve that config with the Monolith/Anomaly DLTX patch dialect. Remembered for the
   *   session, because reopening takes no arguments and has to answer the same descriptors.
   */
  @LatestFlow("spriteImage")
  public *openEquipmentProject(equipmentDdsPath: string, systemLtxPath: string, isDltx: boolean): TFlow {
    this.log.info("Opening equipment project:", equipmentDdsPath, systemLtxPath);

    try {
      this.assetService.releaseKey(SPRITE_ASSET_KEY);
      this.spriteImage = createLoadable(null, true);

      const response: IEquipmentSpriteMetadata = yield* call(
        spriteEquipmentCommands.openSprite(equipmentDdsPath, systemLtxPath, isDltx)
      );

      this.log.info("Equipment project opened:", response);

      const spriteImage: IEquipmentPngDescriptor = yield* call(this.spriteFromResponse(response));

      this.spriteImage = createLoadable(spriteImage);

      yield* this.resolveRepackSource(spriteImage.path);
    } catch (error) {
      this.log.error("Failed to open equipment editor project:", error);

      this.spriteImage = createLoadable(null, false, error as Error);

      emitNotification(this.eventBus, {
        details: `${equipmentDdsPath}\n${transformError(error).message}`,
        severity: ENotificationSeverity.ERROR,
        source: EApplicationGroupId.SPRITES,
        title: "Could not open equipment sprite",
      });
    }
  }

  @LatestFlow("spriteImage")
  public *reopenEquipmentProject(): TFlow {
    yield* this.reopen();
  }

  /**
   * Reads the sprite the backend holds and puts it back on screen.
   *
   * Undecorated on purpose: a repack finishes by reopening, and a decorated call would take the same lane and cancel
   * the repack that made it. Delegating with `yield*` keeps both in one run.
   */
  private *reopen(): TFlow {
    this.log.info("Reopening equipment editor project");

    try {
      this.spriteImage = this.spriteImage.asLoading();

      const response: IEquipmentSpriteMetadata = yield* call(spriteEquipmentCommands.reopenSprite());

      this.log.info("Equipment project reopened:", response);

      const spriteImage: IEquipmentPngDescriptor = yield* call(this.spriteFromResponse(response));

      this.spriteImage = createLoadable(spriteImage);

      yield* this.resolveRepackSource(spriteImage.path);
    } catch (error) {
      this.log.error("Failed to reopen equipment editor project:", error);

      // Left loading, this disables every command in the editor for the rest of the session, and the
      // only way out is closing the project. The previous sprite stays on screen behind the error.
      this.spriteImage = this.spriteImage.asFailed(error as Error);

      throw error;
    }
  }

  @LatestFlow("spriteImage")
  public *repackAndOpenProject(): TFlow {
    const { spriteImage, repackSourcePath } = this;

    if (!spriteImage.value || spriteImage.isLoading) {
      throw new Error("Invalid attempt to reopen project that is loading or not open.");
    }

    if (!repackSourcePath) {
      throw new Error(`Invalid attempt to repack DDS without base icons for '${spriteImage.value.path}'.`);
    }

    this.log.info("Repack and reopen equipment editor project");

    try {
      this.spriteImage = this.spriteImage.asLoading();

      yield* call(
        this.packEquipmentSprite(
          repackSourcePath,
          spriteImage.value.path,
          spriteImage.value.ltxPath,
          spriteImage.value.isDltx
        )
      );

      this.repackedAt = Date.now();

      emitNotification(this.eventBus, {
        details: `${repackSourcePath}\n${spriteImage.value.path}`,
        severity: ENotificationSeverity.SUCCESS,
        source: EApplicationGroupId.SPRITES,
        title: "Repacked equipment sprite",
      });

      yield* this.reopen();
    } catch (error) {
      this.log.error("Failed to repack equipment editor project:", error);

      // Kept as a failure rather than reset to ready. Discarding it here is what made a repack that
      // wrote nothing look exactly like one that succeeded.
      this.spriteImage = this.spriteImage.asFailed(error as Error);

      emitNotification(this.eventBus, {
        details: `${spriteImage.value.path}\n${transformError(error).message}`,
        severity: ENotificationSeverity.ERROR,
        source: EApplicationGroupId.SPRITES,
        title: "Could not repack equipment sprite",
      });

      throw error;
    }
  }

  /**
   * Work out whether this sprite has an unpacked icons directory beside it.
   *
   * The convention is a sibling directory named after the sprite without its extension, which is what the
   * unpacker writes and what the packer reads back.
   *
   * @param spritePath - Path of the open equipment sprite.
   * @returns Resolves whether an unpacked sibling directory is available.
   */
  @BoundAction()
  public *resolveRepackSource(spritePath: string): TFlow {
    try {
      // The directory does not depend on the extension, so both go out together rather than one after the other.
      const [directory, extension] = yield* all([path.dirname(spritePath), path.extname(spritePath)] as const);
      const name: string = yield* call(path.basename(spritePath, extension));
      const sourcePath: string = yield* call(path.join(directory, name));
      const isPresent: boolean = yield* call(exists(sourcePath));

      this.repackSourcePath = isPresent ? sourcePath : null;
    } catch (error) {
      this.log.error("Failed to resolve repack source directory:", error);

      this.repackSourcePath = null;
    }
  }

  @LatestFlow("spriteImage")
  public *closeEquipmentProject(): TFlow {
    this.log.info("Closing equipment project");

    try {
      this.spriteImage = this.spriteImage.asLoading();
      this.assetService.releaseKey(SPRITE_ASSET_KEY);

      yield* call(spriteEquipmentCommands.closeSprite());

      this.log.info("Equipment project closed");

      this.spriteImage = createLoadable(null);
      this.repackSourcePath = null;
      this.repackedAt = null;
    } catch (error) {
      this.log.error("Failed to close equipment editor project:", error);

      this.spriteImage = this.spriteImage.asFailed(new Error(error as string));

      emitNotification(this.eventBus, {
        details: transformError(error).message,
        severity: ENotificationSeverity.ERROR,
        source: EApplicationGroupId.SPRITES,
        title: "Could not close equipment sprite",
      });
    }
  }

  /**
   * Draws every declared icon into one sprite sheet.
   *
   * Started through the jobs service rather than invoked here: reading `system.ltx` pulls in the whole include tree and
   * every icon is decoded, so the run wants an identity, a lease over the sheet it writes, and a way to stop it.
   *
   * @param sourcePath - Directory of individual icon files.
   * @param outputPath - File the sheet is written to.
   * @param systemLtxPath - `system.ltx` declaring which icons exist and where they sit.
   * @param isDltx - Whether to resolve that config with the Monolith/Anomaly DLTX patch dialect.
   * @returns What the run produced.
   */
  public async packEquipmentSprite(
    sourcePath: string,
    outputPath: string,
    systemLtxPath: string,
    isDltx: boolean
  ): Promise<IPackEquipmentResult> {
    this.log.info("Packing equipment editor:", sourcePath, outputPath, systemLtxPath);

    const run: IJobRun<IPackEquipmentResult> = this.jobsService.run<IPackEquipmentResult>({
      kind: EJobKind.SPRITE_EQUIPMENT_PACK,
      invoke: (id: string, progress) =>
        spriteEquipmentCommands.packSprite({ sourcePath, outputPath, systemLtxPath, isDltx }, id, progress),
      describe: (outcome: IJobOutcome<IPackEquipmentResult>): IJobNotice =>
        describePackSpriteOutcome(outputPath, outcome),
    });

    this.packJobId = run.id;

    try {
      return await run.promise;
    } catch (error) {
      this.log.error("Failed to pack equipment editor:", error);
      throw error;
    } finally {
      this.packJobId = null;
    }
  }

  /**
   * @returns The sprite pack currently running, whether this service started it or found it again.
   */
  @Computed()
  public get packJob(): Nullable<IJobState> {
    return this.packJobId
      ? this.jobsService.getJob(this.packJobId)
      : this.jobsService.getJobOfKind(EJobKind.SPRITE_EQUIPMENT_PACK);
  }

  /** Stops the running sprite pack. Nothing has been written yet, so nothing is left behind. */
  @BoundAction()
  public cancelPackEquipmentSprite(): void {
    const job: Nullable<IJobState> = this.packJob;

    if (job) {
      this.jobsService.cancel(job.id);
    }
  }

  public async spriteFromResponse(response: IEquipmentSpriteMetadata): Promise<IEquipmentPngDescriptor> {
    const blob: Blob = await fetch(convertFileSrc(response.name, "stream")).then((response) => response.blob());

    return {
      blob,
      isDltx: response.isDltx,
      ltxPath: response.systemLtxPath,
      descriptors: response.equipmentDescriptors,
      image: await urlToImage(this.assetService.swap(SPRITE_ASSET_KEY, blob)),
      name: response.name,
      path: response.path,
    };
  }
}
