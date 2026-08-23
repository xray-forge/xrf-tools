import { clamp } from "@mui/x-data-grid/internals";
import { path } from "@tauri-apps/api";
import { convertFileSrc } from "@tauri-apps/api/core";
import { exists } from "@tauri-apps/plugin-fs";
import { EventBus, inject, Injectable, OnDeactivation, OnProvision } from "@wirestate/core";
import { BoundAction, makeObservable, Observable, runInAction } from "@wirestate/mobx";

import { urlToImage } from "@/core/assets/image";
import { AssetService } from "@/core/assets/services";
import { equipmentIconsCommands } from "@/core/bindings/commands/equipment-icons";
import {
  IEquipmentSectionDescriptor,
  IEquipmentSpriteMetadata,
  IPackEquipmentResult,
} from "@/core/equipment-icons/equipment";
import { transformError } from "@/core/error/lib";
import { releaseEditorProject } from "@/core/ipc/release";
import { emitNotification, ENotificationSeverity } from "@/core/notifications/lib";
import { EApplicationGroupId } from "@/core/routing/application";
import { createLoadable, Loadable } from "@/lib/loadable";
import { Logger } from "@/lib/logging";
import { Nullable } from "@/lib/types/general";

export interface IEquipmentPngDescriptor {
  ltxPath: string;
  descriptors: Array<IEquipmentSectionDescriptor>;
  path: string;
  name: string;
  blob: Blob;
  image: HTMLImageElement;
}

/** One sprite is open at a time, so its url lives under a fixed key rather than being tracked by hand. */
const SPRITE_ASSET_KEY: string = "equipment-sprite";

@Injectable()
export class EquipmentService {
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

  public constructor(
    private readonly assetService: AssetService = inject(AssetService),
    private readonly eventBus: EventBus = inject(EventBus)
  ) {
    makeObservable(this);
  }

  @OnProvision()
  public async onProvision(): Promise<void> {
    const response: Nullable<IEquipmentSpriteMetadata> = await equipmentIconsCommands.getSprite();

    if (response) {
      this.log.info("Existing equipment sprite detected");
      runInAction(() => (this.isReady = true));

      const spriteImage: IEquipmentPngDescriptor = await this.spriteFromResponse(response);

      runInAction(() => (this.spriteImage = createLoadable(spriteImage)));

      await this.resolveRepackSource(spriteImage.path);
    } else {
      this.log.info("No existing sprite detected file");
      runInAction(() => (this.isReady = true));
    }
  }

  /**
   * Release the sprite when the editor is navigated away from.
   */
  @OnDeactivation()
  public onDeactivation(): void {
    this.assetService.releaseKey(SPRITE_ASSET_KEY);
    releaseEditorProject(equipmentIconsCommands.closeSprite);
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

  @BoundAction()
  public async openEquipmentProject(equipmentDdsPath: string, systemLtxPath: string): Promise<void> {
    this.log.info("Opening equipment project:", equipmentDdsPath, systemLtxPath);

    try {
      this.assetService.releaseKey(SPRITE_ASSET_KEY);
      this.spriteImage = createLoadable(null, true);

      const response: IEquipmentSpriteMetadata = await equipmentIconsCommands.openSprite(
        equipmentDdsPath,
        systemLtxPath
      );

      this.log.info("Equipment project opened:", response);

      const spriteImage: IEquipmentPngDescriptor = await this.spriteFromResponse(response);

      runInAction(() => (this.spriteImage = createLoadable(spriteImage)));

      await this.resolveRepackSource(spriteImage.path);
    } catch (error) {
      this.log.error("Failed to open equipment editor project:", error);

      runInAction(() => (this.spriteImage = createLoadable(null, false, error as Error)));

      emitNotification(this.eventBus, {
        details: `${equipmentDdsPath}\n${transformError(error).message}`,
        severity: ENotificationSeverity.ERROR,
        source: EApplicationGroupId.ICONS,
        title: "Could not open equipment sprite",
      });
    }
  }

  @BoundAction()
  public async reopenEquipmentProject(): Promise<void> {
    this.log.info("Reopening equipment editor project");

    try {
      this.spriteImage = this.spriteImage.asLoading();

      const response: IEquipmentSpriteMetadata = await equipmentIconsCommands.reopenSprite();

      this.log.info("Equipment project reopened:", response);

      const spriteImage: IEquipmentPngDescriptor = await this.spriteFromResponse(response);

      runInAction(() => (this.spriteImage = createLoadable(spriteImage)));

      await this.resolveRepackSource(spriteImage.path);
    } catch (error) {
      this.log.error("Failed to reopen equipment editor project:", error);

      // Left loading, this disables every command in the editor for the rest of the session, and the
      // only way out is closing the project. The previous sprite stays on screen behind the error.
      runInAction(() => (this.spriteImage = this.spriteImage.asFailed(error as Error)));

      throw error;
    }
  }

  @BoundAction()
  public async repackAndOpenProject(): Promise<void> {
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

      await this.packEquipmentSprite(repackSourcePath, spriteImage.value.path, spriteImage.value.ltxPath);

      runInAction(() => (this.repackedAt = Date.now()));

      emitNotification(this.eventBus, {
        details: `${repackSourcePath}\n${spriteImage.value.path}`,
        severity: ENotificationSeverity.SUCCESS,
        source: EApplicationGroupId.ICONS,
        title: "Repacked equipment sprite",
      });

      await this.reopenEquipmentProject();
    } catch (error) {
      this.log.error("Failed to repack equipment editor project:", error);

      // Kept as a failure rather than reset to ready. Discarding it here is what made a repack that
      // wrote nothing look exactly like one that succeeded.
      runInAction(() => (this.spriteImage = this.spriteImage.asFailed(error as Error)));

      emitNotification(this.eventBus, {
        details: `${spriteImage.value.path}\n${transformError(error).message}`,
        severity: ENotificationSeverity.ERROR,
        source: EApplicationGroupId.ICONS,
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
  public async resolveRepackSource(spritePath: string): Promise<void> {
    try {
      const sourcePath: string = await path.join(
        await path.dirname(spritePath),
        await path.basename(spritePath, await path.extname(spritePath))
      );

      const isPresent: boolean = await exists(sourcePath);

      runInAction(() => (this.repackSourcePath = isPresent ? sourcePath : null));
    } catch (error) {
      this.log.error("Failed to resolve repack source directory:", error);

      runInAction(() => (this.repackSourcePath = null));
    }
  }

  @BoundAction()
  public async closeEquipmentProject(): Promise<void> {
    this.log.info("Closing equipment project");

    try {
      this.spriteImage = this.spriteImage.asLoading();
      this.assetService.releaseKey(SPRITE_ASSET_KEY);

      await equipmentIconsCommands.closeSprite();

      this.log.info("Equipment project closed");

      runInAction(() => {
        this.spriteImage = createLoadable(null);
        this.repackSourcePath = null;
        this.repackedAt = null;
      });
    } catch (error) {
      this.log.error("Failed to close equipment editor project:", error);
      runInAction(() => (this.spriteImage = this.spriteImage.asFailed(new Error(error as string))));

      emitNotification(this.eventBus, {
        details: transformError(error).message,
        severity: ENotificationSeverity.ERROR,
        source: EApplicationGroupId.ICONS,
        title: "Could not close equipment sprite",
      });
    }
  }

  public async packEquipmentSprite(
    sourcePath: string,
    outputPath: string,
    systemLtxPath: string
  ): Promise<IPackEquipmentResult> {
    this.log.info("Packing equipment editor:", sourcePath, outputPath, systemLtxPath);

    try {
      return await equipmentIconsCommands.packSprite(sourcePath, outputPath, systemLtxPath);
    } catch (error) {
      this.log.error("Failed to pack equipment editor:", error);
      throw error;
    }
  }

  public async spriteFromResponse(response: IEquipmentSpriteMetadata): Promise<IEquipmentPngDescriptor> {
    const blob: Blob = await fetch(convertFileSrc(response.name, "stream")).then((response) => response.blob());

    return {
      blob,
      ltxPath: response.systemLtxPath,
      descriptors: response.equipmentDescriptors,
      image: await urlToImage(this.assetService.swap(SPRITE_ASSET_KEY, blob)),
      name: response.name,
      path: response.path,
    };
  }
}
