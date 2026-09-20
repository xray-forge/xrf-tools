import { path } from "@tauri-apps/api";
import { convertFileSrc } from "@tauri-apps/api/core";
import { exists } from "@tauri-apps/plugin-fs";
import { EventBus, inject, Injectable, OnDeactivation, OnProvision } from "@wirestate/core";
import { BoundAction, Computed, flowResult, Observable } from "@wirestate/mobx";

import { RELOAD_EQUIPMENT_SPRITE_KEYBIND_COMMAND } from "@/applications/sprite-equipment-editor/commands";
import { urlToImage } from "@/core/assets/lib/image";
import { AssetService } from "@/core/assets/services";
import { KeybindCommand } from "@/core/commands";
import { transformError } from "@/core/error/lib";
import { spriteEquipmentCommands } from "@/core/ipc/commands/sprite-equipment";
import { requireSessionId, Session } from "@/core/ipc/session";
import {
  EEquipmentConfigSource,
  EquipmentConfigSource,
  EquipmentSpriteMetadata,
  EquipmentSpriteOpen,
  SessionSnapshot,
} from "@/core/ipc/types/xrf-app";
import { PackEquipmentResult } from "@/core/ipc/types/xrf-texture";
import { emitNotification, ENotificationSeverity } from "@/core/notifications/lib";
import { EApplicationGroupId } from "@/core/routing/application";
import { describeEquipmentSheet, ENGINE_GRID_SQUARE } from "@/core/sprite-equipment/lib";
import { SpriteEquipmentPackerService } from "@/core/sprite-equipment/services/packer";
import { AsyncState } from "@/lib/async-state";
import { formatDuration } from "@/lib/format/duration";
import { Logger, Timer } from "@/lib/logging";
import { all, call, cancelFlow, ExclusiveFlow, LatestFlow, TFlow } from "@/lib/mobx";
import { Nullable } from "@/lib/types/general";

/**
 * The sheet on screen: what the backend answered, and the image decoded from it.
 */
export interface IOpenEquipmentSprite {
  sessionId: string;
  metadata: EquipmentSpriteMetadata;
  image: HTMLImageElement;
}

/** The two filesystem paths a repack needs, which only an open over loose files can supply. */
export interface IEquipmentRepackTargets {
  /** The sheet to overwrite. */
  sheet: string;
  /** The configuration declaring what goes on it. */
  config: string;
}

/** The open sprite, its image lifetime, and editor actions. */
@Injectable()
export class SpriteEquipmentEditorService {
  public readonly log: Logger = new Logger(__MODULE_NAME__);

  private readonly session: Session = new Session(spriteEquipmentCommands.closeSprite);

  @Observable()
  public isReady: boolean = false;

  @Observable()
  public isGridVisible: boolean = true;

  /**
   * Whether the slots the configuration claims are shaded over the picture.
   */
  @Observable()
  public isOccupancyVisible: boolean = true;

  @Observable()
  public gridSize: number = ENGINE_GRID_SQUARE;

  @Observable()
  public spriteImage: AsyncState<IOpenEquipmentSprite> = AsyncState.idle();

  /**
   * Directory the sprite can be rebuilt from, or null when there is nothing to rebuild from.
   */
  @Observable()
  public repackSourcePath: Nullable<string> = null;

  /** Timestamp of the last successful repack, so the status bar can confirm the write happened. */
  @Observable()
  public repackedAt: Nullable<number> = null;

  public constructor(
    private readonly assetService: AssetService = inject(AssetService),
    private readonly eventBus: EventBus = inject(EventBus),
    private readonly packerService: SpriteEquipmentPackerService = inject(SpriteEquipmentPackerService)
  ) {}

  /**
   * The paths a repack would write from and to, or null when this open cannot be repacked.
   */
  @Computed()
  public get repackTargets(): Nullable<IEquipmentRepackTargets> {
    const metadata: Nullable<EquipmentSpriteMetadata> = this.spriteImage.value?.metadata ?? null;

    if (!metadata) {
      return null;
    }

    const config: Nullable<EquipmentConfigSource> = metadata.open.config;
    const sheet: Nullable<string> = metadata.location.path;

    return sheet && config?.kind === EEquipmentConfigSource.FILE ? { sheet, config: config.path } : null;
  }

  @OnProvision()
  public async onProvision(): Promise<void> {
    try {
      await flowResult(this.restore());
    } catch (error: unknown) {
      this.log.error("Failed to restore equipment sprite:", error);

      throw error;
    }
  }

  /**
   * Release the sprite when the editor is navigated away from.
   */
  @OnDeactivation()
  public onDeactivation(): void {
    cancelFlow(this, "spriteImage");

    this.assetService.release(this.spriteImage.value?.image.src ?? null);

    this.session.release();
  }

  /**
   * Restores the committed session without superseding a user action in the same flow.
   */
  @ExclusiveFlow("spriteImage")
  private *restore(): TFlow {
    const response: Nullable<SessionSnapshot<EquipmentSpriteMetadata>> = yield* call(
      spriteEquipmentCommands.getSprite()
    );

    if (!response) {
      this.log.info("No existing sprite detected file");
      this.spriteImage = this.spriteImage.asReady(null);
      this.isReady = true;

      return;
    }

    this.log.info("Existing equipment sprite detected");
    this.session.adopt(response);
    this.isReady = true;

    yield* this.viewSprite(response);

    this.log.info(
      "Equipment sprite restored:",
      response.value.location.path,
      response.value.occupants.length,
      "occupants"
    );
  }

  @BoundAction()
  public setGridVisibility(isVisible: boolean): void {
    this.isGridVisible = isVisible;
  }

  @BoundAction()
  public setOccupancyVisibility(isVisible: boolean): void {
    this.isOccupancyVisible = isVisible;
  }

  /**
   * Dismisses a reported failure while keeping the displayed sprite.
   */
  @BoundAction()
  public clearSpriteError(): void {
    if (this.spriteImage.isFailed) {
      this.spriteImage = this.spriteImage.value === null ? this.spriteImage.asIdle() : this.spriteImage.asReady();
    }
  }

  @BoundAction()
  public setGridSize(size: number): void {
    this.gridSize = Math.round(Math.min(100, Math.max(10, size)));
  }

  /**
   * Reads a sprite sheet and the configuration naming its icons.
   *
   * @param open - What to read: the trees to search, the sheet, and the configuration that annotates it. Kept by the
   *   backend so a reload repeats the request rather than the paths it happened to resolve to.
   */
  @LatestFlow("spriteImage")
  public *openEquipmentProject(open: EquipmentSpriteOpen): TFlow {
    const timer: Timer = new Timer();

    this.log.info("Opening equipment project:", describeEquipmentSheet(open.sheet));

    try {
      this.spriteImage = this.spriteImage.asLoading();

      const response: SessionSnapshot<EquipmentSpriteMetadata> = yield* call(
        this.session.open((sessionId) => spriteEquipmentCommands.openSprite({ sessionId, ...open }))
      );

      yield* this.viewSprite(response);

      this.log.info(
        "Equipment project opened:",
        response.value.location.path,
        response.value.occupants.length,
        "occupants, in",
        formatDuration(timer.elapsed())
      );
    } catch (error) {
      this.log.error(
        "Failed to open equipment editor project:",
        describeEquipmentSheet(open.sheet),
        "after",
        formatDuration(timer.elapsed()),
        error
      );

      this.spriteImage = this.spriteImage.asFailed(error as Error);

      emitNotification(this.eventBus, {
        details: `${describeEquipmentSheet(open.sheet)}\n${transformError(error).message}`,
        severity: ENotificationSeverity.ERROR,
        source: EApplicationGroupId.SPRITES,
        title: "Could not open equipment sprite",
      });
    }
  }

  /**
   * Reads the open sprite again, from the toolbar or from `F5`.
   */
  @KeybindCommand(RELOAD_EQUIPMENT_SPRITE_KEYBIND_COMMAND, {
    isEnabled: (service: SpriteEquipmentEditorService) =>
      service.spriteImage.value !== null && !service.spriteImage.isLoading,
  })
  @LatestFlow("spriteImage")
  public *reopenEquipmentProject(): TFlow {
    yield* this.reopen();
  }

  /**
   * Reads the sprite the backend holds and puts it back on screen.
   */
  private *reopen(): TFlow {
    const timer: Timer = new Timer();

    this.log.info("Reopening equipment editor project");

    try {
      this.spriteImage = this.spriteImage.asLoading();

      const response: SessionSnapshot<EquipmentSpriteMetadata> = yield* call(
        this.session.open((openingId) =>
          spriteEquipmentCommands.reopenSprite(requireSessionId(this.spriteImage.value), openingId)
        )
      );

      yield* this.viewSprite(response);

      this.log.info(
        "Equipment project reopened:",
        response.value.location.path,
        response.value.occupants.length,
        "occupants, in",
        formatDuration(timer.elapsed())
      );
    } catch (error) {
      this.log.error("Failed to reopen equipment editor project:", "after", formatDuration(timer.elapsed()), error);

      // Left loading, this disables every command in the editor for the rest of the session, and the
      // only way out is closing the project. The previous sprite stays on screen behind the error.
      this.spriteImage = this.spriteImage.asFailed(error as Error);

      throw error;
    }
  }

  @ExclusiveFlow("spriteImage")
  public *repackAndOpenProject(): TFlow {
    const { spriteImage: spriteImage, repackSourcePath } = this;

    if (!spriteImage.value || spriteImage.isLoading) {
      throw new Error("Invalid attempt to reopen project that is loading or not open.");
    }

    const targets: Nullable<IEquipmentRepackTargets> = this.repackTargets;

    // Two different refusals, kept apart because they have two different fixes: unpack the sheet first, or reopen it
    // from files a repack can actually write.
    if (!targets) {
      throw new Error("Invalid attempt to repack a sheet that was not opened from files on disk.");
    }

    if (!repackSourcePath) {
      throw new Error(`Invalid attempt to repack DDS without base icons for '${targets.sheet}'.`);
    }

    this.log.info("Repack and reopen equipment editor project");

    try {
      this.spriteImage = this.spriteImage.asLoading();

      const result: Nullable<PackEquipmentResult> = yield* call(
        flowResult(
          this.packerService.packEquipmentSprite(
            repackSourcePath,
            targets.sheet,
            targets.config,
            spriteImage.value.metadata.open.isDltx
          )
        )
      );

      if (!result || result.outcome !== "completed") {
        this.spriteImage = spriteImage;

        return;
      }

      this.repackedAt = Date.now();

      emitNotification(this.eventBus, {
        details: `${repackSourcePath}\n${targets.sheet}`,
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
        details: `${targets.sheet}\n${transformError(error).message}`,
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
   * @param spritePath - Path of the open equipment sprite.
   * @returns Resolves whether an unpacked sibling directory is available.
   */
  @BoundAction()
  private *resolveRepackSource(spritePath: Nullable<string>): TFlow {
    if (!spritePath) {
      // An archived sheet has no directory to look beside, which is not a failure - it is a sheet nothing can repack.
      this.repackSourcePath = null;

      return;
    }

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

      yield* call(this.session.close());

      this.assetService.release(this.spriteImage.value?.image.src ?? null);

      this.log.info("Equipment project closed");

      this.spriteImage = this.spriteImage.asIdle();
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
   * Publishes a decoded sprite and releases candidates abandoned by cancellation.
   *
   * @param response - Native snapshot whose preview should be displayed.
   */
  private *viewSprite(response: SessionSnapshot<EquipmentSpriteMetadata>): TFlow {
    const pending: Promise<IOpenEquipmentSprite> = this.spriteFromResponse(response);

    let published: boolean = false;

    try {
      const spriteImage: IOpenEquipmentSprite = yield* call(pending);

      this.assetService.release(this.spriteImage.value?.image.src ?? null);
      this.spriteImage = this.spriteImage.asReady(spriteImage);
      published = true;
      yield* this.resolveRepackSource(spriteImage.metadata.location.path);
    } finally {
      if (!published) {
        void pending.then(
          (sprite) => this.assetService.release(sprite.image.src),
          () => undefined
        );
      }
    }
  }

  private async spriteFromResponse(response: SessionSnapshot<EquipmentSpriteMetadata>): Promise<IOpenEquipmentSprite> {
    const { sessionId, value: metadata } = response;

    const preview: Response = await fetch(convertFileSrc(sessionId + "/" + metadata.name, "stream"));
    const url: string = this.assetService.create(await preview.blob());

    try {
      return { sessionId, metadata, image: await urlToImage(url) };
    } catch (error) {
      this.assetService.release(url);
      throw error;
    }
  }
}
