import { path } from "@tauri-apps/api";
import { convertFileSrc } from "@tauri-apps/api/core";
import { exists } from "@tauri-apps/plugin-fs";
import { EventBus, inject, Injectable, OnDeactivation, OnProvision } from "@wirestate/core";
import { BoundAction, flowResult, Observable } from "@wirestate/mobx";

import { urlToImage } from "@/core/assets/image";
import { AssetService } from "@/core/assets/services";
import { spriteEquipmentCommands } from "@/core/bindings/commands/sprite-equipment";
import { SessionSnapshot } from "@/core/bindings/types/xrf-app";
import { transformError } from "@/core/error/lib";
import { releaseEditorProject } from "@/core/ipc/release";
import { requireSessionId, Session } from "@/core/ipc/session";
import { emitNotification, ENotificationSeverity } from "@/core/notifications/lib";
import { EApplicationGroupId } from "@/core/routing/application";
import {
  IEquipmentSectionDescriptor,
  IEquipmentSpriteMetadata,
  IPackEquipmentResult,
} from "@/core/sprite-equipment/equipment";
import { SpriteEquipmentPackerService } from "@/core/sprite-equipment/services/packer";
import { AsyncState } from "@/lib/async-state";
import { Logger } from "@/lib/logging";
import { all, call, cancelFlow, ExclusiveFlow, LatestFlow, TFlow } from "@/lib/mobx";
import { Nullable } from "@/lib/types/general";

export interface IEquipmentPngDescriptor {
  sessionId: string;
  ltxPath: string;
  /** Whether the open project's descriptors came out of a DLTX-resolved config tree. */
  isDltx: boolean;
  descriptors: Array<IEquipmentSectionDescriptor>;
  path: string;
  name: string;
  blob: Blob;
  image: HTMLImageElement;
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

  @Observable()
  public gridSize: number = 50;

  @Observable()
  public spriteImage: AsyncState<IEquipmentPngDescriptor> = AsyncState.idle();

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

  @OnProvision()
  public async onProvision(): Promise<void> {
    await flowResult(this.restore());
  }

  /**
   * Release the sprite when the editor is navigated away from.
   */
  @OnDeactivation()
  public onDeactivation(): void {
    cancelFlow(this, "spriteImage");

    this.assetService.release(this.spriteImage.value?.image.src ?? null);

    releaseEditorProject(() => this.session.close(this.spriteImage.value?.sessionId));
  }

  /**
   * Restores the committed session without superseding a user action in the same flow.
   */
  @ExclusiveFlow("spriteImage")
  private *restore(): TFlow {
    const response: Nullable<SessionSnapshot<IEquipmentSpriteMetadata>> = yield* call(
      spriteEquipmentCommands.getSprite()
    );

    if (!response) {
      this.log.info("No existing sprite detected file");
      this.spriteImage = this.spriteImage.asReady(null);
      this.isReady = true;

      return;
    }

    this.log.info("Existing equipment sprite detected");
    this.isReady = true;

    yield* this.viewSprite(response);
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
   * @param equipmentDdsPath - The packed `*.dds` holding the inventory icons.
   * @param systemLtxPath - `system.ltx` declaring which icons exist and where they sit.
   * @param isDltx - Whether to resolve that config with the Monolith/Anomaly DLTX patch dialect. Remembered for the
   *   session, so reopening resolves the same descriptors.
   */
  @LatestFlow("spriteImage")
  public *openEquipmentProject(equipmentDdsPath: string, systemLtxPath: string, isDltx: boolean): TFlow {
    this.log.info("Opening equipment project:", equipmentDdsPath, systemLtxPath);

    try {
      this.spriteImage = this.spriteImage.asLoading();

      const response: SessionSnapshot<IEquipmentSpriteMetadata> = yield* call(
        this.session.open((sessionId) =>
          spriteEquipmentCommands.openSprite({ sessionId, equipmentDdsPath, systemLtxPath, isDltx })
        )
      );

      this.log.info("Equipment project opened:", response);

      yield* this.viewSprite(response);
    } catch (error) {
      this.log.error("Failed to open equipment editor project:", error);

      this.spriteImage = this.spriteImage.asFailed(error as Error);

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

      const response: SessionSnapshot<IEquipmentSpriteMetadata> = yield* call(
        this.session.open((openingId) =>
          spriteEquipmentCommands.reopenSprite(requireSessionId(this.spriteImage.value), openingId)
        )
      );

      this.log.info("Equipment project reopened:", response);

      yield* this.viewSprite(response);
    } catch (error) {
      this.log.error("Failed to reopen equipment editor project:", error);

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

    if (!repackSourcePath) {
      throw new Error(`Invalid attempt to repack DDS without base icons for '${spriteImage.value.path}'.`);
    }

    this.log.info("Repack and reopen equipment editor project");

    try {
      this.spriteImage = this.spriteImage.asLoading();

      const result: Nullable<IPackEquipmentResult> = yield* call(
        flowResult(
          this.packerService.packEquipmentSprite(
            repackSourcePath,
            spriteImage.value.path,
            spriteImage.value.ltxPath,
            spriteImage.value.isDltx
          )
        )
      );

      if (!result || result.outcome !== "completed") {
        this.spriteImage = spriteImage;

        return;
      }

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
  private *resolveRepackSource(spritePath: string): TFlow {
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

      yield* call(this.session.close(this.spriteImage.value?.sessionId));

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
  private *viewSprite(response: SessionSnapshot<IEquipmentSpriteMetadata>): TFlow {
    const pending: Promise<IEquipmentPngDescriptor> = this.spriteFromResponse(response);

    let published: boolean = false;

    try {
      const spriteImage: IEquipmentPngDescriptor = yield* call(pending);

      this.assetService.release(this.spriteImage.value?.image.src ?? null);
      this.spriteImage = this.spriteImage.asReady(spriteImage);
      published = true;
      yield* this.resolveRepackSource(spriteImage.path);
    } finally {
      if (!published) {
        void pending.then(
          (sprite) => this.assetService.release(sprite.image.src),
          () => undefined
        );
      }
    }
  }

  private async spriteFromResponse(
    response: SessionSnapshot<IEquipmentSpriteMetadata>
  ): Promise<IEquipmentPngDescriptor> {
    const { sessionId, value: metadata } = response;

    const preview: Response = await fetch(convertFileSrc(sessionId + "/" + metadata.name, "stream"));
    const blob: Blob = await preview.blob();

    const url: string = this.assetService.create(blob);

    try {
      return {
        sessionId,
        blob,
        isDltx: metadata.isDltx,
        ltxPath: metadata.systemLtxPath,
        descriptors: metadata.equipmentDescriptors,
        image: await urlToImage(url),
        name: metadata.name,
        path: metadata.path,
      };
    } catch (error) {
      this.assetService.release(url);
      throw error;
    }
  }
}
