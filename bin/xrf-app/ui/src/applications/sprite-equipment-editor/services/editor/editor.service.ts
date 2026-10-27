import { path } from "@tauri-apps/api";
import { exists } from "@tauri-apps/plugin-fs";
import { inject, Injectable, OnDeactivation, OnProvision } from "@wirestate/core";
import { BoundAction, Computed, flowResult, Observable } from "@wirestate/mobx";
import { AsyncState } from "@xrf/async";
import { all, call, ExclusiveFlow, TFlow } from "@xrf/flow";
import { fetchBulk } from "@xrf/ipc";
import { spriteEquipmentCommands } from "@xrf/ipc/commands/sprite-equipment";
import { spriteEquipmentBulkRoutes } from "@xrf/ipc/commands/sprite-equipment-bulk";
import { AssetSource, EAssetSource, SessionId, SessionSnapshot } from "@xrf/ipc/types/xrf-app-core";
import { EquipmentSpriteMetadata, EquipmentSpriteOpen } from "@xrf/ipc/types/xrf-app-sprite-equipment";
import { EJobOutcome } from "@xrf/ipc/types/xrf-job";
import { EquipmentSpritePackResult } from "@xrf/ipc/types/xrf-texture";
import { Logger } from "@xrf/logging";
import { clamp } from "@xrf/math";
import { Nullable } from "@xrf/types";

import { RELOAD_EQUIPMENT_SPRITE_KEYBIND_COMMAND } from "@/applications/sprite-equipment-editor/commands";
import { urlToImage } from "@/core/assets/lib/image";
import { AssetService } from "@/core/assets/services";
import { KeybindCommand } from "@/core/commands";
import { CommandOperation } from "@/core/operations/lib/command-operation";
import { TCommandCompletion } from "@/core/operations/lib/command-reporter";
import { SessionRootResource } from "@/core/session/lib/session-root-resource";
import { describeEquipmentSheet, ENGINE_GRID_SQUARE } from "@/core/sprite-equipment/lib";
import { SpriteEquipmentPackerService } from "@/core/sprite-equipment/services/packer";
import { noop } from "@/lib/callbacks/noop";

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

  /** The sheet the backend holds open, and what it says of it. */
  public readonly sprite: SessionRootResource<EquipmentSpriteMetadata> = new SessionRootResource(this.log, {
    subject: "equipment sprite",
    close: spriteEquipmentCommands.closeSprite,
    restore: spriteEquipmentCommands.restoreSprite,
    adopted: (snapshot: SessionSnapshot<EquipmentSpriteMetadata>) => this.decode(snapshot),
  });

  /** The sheet's picture, decoded for each sheet adopted; the last one stays on screen while the next decodes. */
  public readonly picture: CommandOperation<IOpenEquipmentSprite> = new CommandOperation(this.log);

  /** The last repack, which a person is told about and whose failure the editor keeps showing. */
  public readonly repack: CommandOperation<Nullable<EquipmentSpritePackResult>> = new CommandOperation(this.log);

  @Observable()
  public isGridVisible: boolean = true;

  /**
   * Whether the slots the configuration claims are shaded over the picture.
   */
  @Observable()
  public isOccupancyVisible: boolean = true;

  @Observable()
  public gridSize: number = ENGINE_GRID_SQUARE;

  /**
   * Directory the sprite can be rebuilt from, or null when there is nothing to rebuild from.
   */
  @Observable()
  public repackSourcePath: Nullable<string> = null;

  /** Timestamp of the last successful repack, so the status bar can confirm the write happened. */
  @Observable()
  public repackedAt: Nullable<number> = null;

  /** The failure a person dismissed, shown no longer while it is still the latest. */
  @Observable()
  private dismissed: Nullable<Error> = null;

  public constructor(
    private readonly assetService: AssetService = inject(AssetService),
    private readonly packerService: SpriteEquipmentPackerService = inject(SpriteEquipmentPackerService)
  ) {}

  /** Whether the backend has been asked what it still holds, which holds the loader back. */
  @Computed()
  public get isReady(): boolean {
    return this.sprite.isRestored;
  }

  /**
   * The sheet as the editor shows it: its picture while one is held, loading while the sheet, its picture or a repack
   * is, and the latest failure of the three a person has not dismissed.
   */
  @Computed()
  public get spriteImage(): AsyncState<IOpenEquipmentSprite> {
    const value: Nullable<IOpenEquipmentSprite> = this.sprite.value ? this.picture.value : null;
    const failures: Array<Nullable<Error>> = [this.repack.error, this.sprite.error, this.picture.error];
    const error: Nullable<Error> =
      failures.find((failure: Nullable<Error>) => failure !== null && failure !== this.dismissed) ?? null;

    if (this.sprite.isLoading || this.picture.isLoading || this.repack.isLoading) {
      return AsyncState.loading(value);
    }

    if (error) {
      return AsyncState.failed(error, value);
    }

    return value ? AsyncState.ready(value) : AsyncState.idle();
  }

  /**
   * The paths a repack would write from and to, or null when this open cannot be repacked.
   */
  @Computed()
  public get repackTargets(): Nullable<IEquipmentRepackTargets> {
    const metadata: Nullable<EquipmentSpriteMetadata> = this.sprite.value;

    if (!metadata) {
      return null;
    }

    const config: Nullable<AssetSource> = metadata.open.config;
    const sheet: Nullable<string> = metadata.location.path;

    return sheet && config?.kind === EAssetSource.FILE ? { sheet, config: config.path } : null;
  }

  @OnProvision()
  public async onProvision(): Promise<void> {
    await flowResult(this.sprite.restore());
  }

  /**
   * Release the sprite's image when the editor is navigated away from; the tool's scope closes the sprite.
   */
  @OnDeactivation()
  public onDeactivation(): void {
    this.assetService.release(this.picture.value?.image.src ?? null);
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
    this.dismissed = this.spriteImage.error;
  }

  @BoundAction()
  public setGridSize(size: number): void {
    this.gridSize = Math.round(clamp(size, 10, 100));
  }

  /**
   * Reads a sprite sheet and the configuration naming its icons.
   *
   * @param open - What to read: the trees to search, the sheet, and the configuration that annotates it. Kept by the
   *   backend so a reload repeats the request rather than the paths it happened to resolve to.
   * @returns Whether the sheet opened.
   */
  @BoundAction()
  public openEquipmentProject(open: EquipmentSpriteOpen): Promise<boolean> {
    return flowResult(
      this.sprite.open(describeEquipmentSheet(open.sheet), (sessionId: SessionId) =>
        spriteEquipmentCommands.openSprite({ sessionId, ...open })
      )
    );
  }

  /**
   * Reads the open sprite again, from the toolbar or from `F5`.
   *
   * @returns Whether the sheet was read again.
   */
  @KeybindCommand(RELOAD_EQUIPMENT_SPRITE_KEYBIND_COMMAND, {
    isEnabled: (service: SpriteEquipmentEditorService) =>
      service.spriteImage.value !== null && !service.spriteImage.isLoading,
  })
  @BoundAction()
  public reopenEquipmentProject(): Promise<boolean> {
    return this.reopen();
  }

  /**
   * Rebuilds the sheet from the icons unpacked beside it, then reads it again.
   *
   * @throws {Error} When nothing is open, or the sheet cannot be repacked: refusals the action never offers.
   */
  @ExclusiveFlow("repack")
  public *repackAndOpenProject(): TFlow {
    const { repackSourcePath, spriteImage } = this;
    const metadata: Nullable<EquipmentSpriteMetadata> = this.sprite.value;

    if (!metadata || spriteImage.isLoading) {
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

    const completion: TCommandCompletion<Nullable<EquipmentSpritePackResult>> = yield* this.repack.run({
      name: "Repack equipment sprite",
      subject: targets.sheet,
      command: () =>
        flowResult(
          this.packerService.packEquipmentSprite(repackSourcePath, targets.sheet, targets.config, metadata.open.isDltx)
        ),
      failure: "Could not repack equipment sprite",
      // A busy or cancelled repack wrote nothing, so it says nothing and the sheet is not read again.
      success: (result: Nullable<EquipmentSpritePackResult>) =>
        result?.outcome === EJobOutcome.COMPLETED
          ? { details: `${repackSourcePath}\n${targets.sheet}`, title: "Repacked equipment sprite" }
          : null,
    });

    if (completion.error) {
      throw completion.error;
    }

    if (completion.result?.outcome === EJobOutcome.COMPLETED) {
      this.repackedAt = Date.now();

      yield* call(this.reopen());
    }
  }

  /**
   * Lets the sheet go, and its picture with it.
   */
  @BoundAction()
  public async closeEquipmentProject(): Promise<void> {
    const image: Nullable<string> = this.picture.value?.image.src ?? null;

    if (await flowResult(this.sprite.close())) {
      this.forget(image);
    }
  }

  @BoundAction()
  private forget(image: Nullable<string>): void {
    this.assetService.release(image);
    this.picture.reset();
    this.repack.reset();
    this.repackSourcePath = null;
    this.repackedAt = null;
  }

  /** Reads the sheet the backend holds again, in a fresh opening the held one is replaced by. */
  private reopen(): Promise<boolean> {
    const held: Nullable<SessionSnapshot<EquipmentSpriteMetadata>> = this.sprite.snapshot;

    if (!held) {
      return Promise.resolve(false);
    }

    return flowResult(
      this.sprite.refresh(describeEquipmentSheet(held.value.open.sheet), (sessionId: SessionId) =>
        spriteEquipmentCommands.reloadSprite(sessionId, held.sessionId)
      )
    );
  }

  /**
   * Decodes an adopted sheet's picture, releasing the one it replaces once it shows and any a newer sheet abandoned.
   *
   * @param snapshot - The sheet the backend answered.
   */
  private *decode(snapshot: SessionSnapshot<EquipmentSpriteMetadata>): TFlow {
    const held: Nullable<IOpenEquipmentSprite> = this.picture.value;
    const pending: Promise<IOpenEquipmentSprite> = this.spriteFromResponse(snapshot);

    let published: boolean = false;

    try {
      const completion: TCommandCompletion<IOpenEquipmentSprite> = yield* this.picture.run({
        name: "Decode equipment sprite",
        subject: describeEquipmentSheet(snapshot.value.open.sheet),
        command: () => pending,
        failure: "Could not decode the equipment sprite",
      });

      if (completion.error === null) {
        published = true;
        this.assetService.release(held?.image.src ?? null);

        yield* this.resolveRepackSource(completion.result.metadata.location.path);
      }
    } finally {
      if (!published) {
        void pending.then((sprite: IOpenEquipmentSprite) => this.assetService.release(sprite.image.src), noop);
      }
    }
  }

  /**
   * Work out whether this sprite has an unpacked icons directory beside it.
   *
   * @param spritePath - Path of the open equipment sprite.
   */
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

  private async spriteFromResponse(response: SessionSnapshot<EquipmentSpriteMetadata>): Promise<IOpenEquipmentSprite> {
    const { sessionId, value: metadata } = response;

    const preview: ArrayBuffer = await fetchBulk(spriteEquipmentBulkRoutes.readPreview(sessionId));
    const url: string = this.assetService.create(new Blob([preview], { type: "image/png" }));

    try {
      return { sessionId, metadata, image: await urlToImage(url) };
    } catch (error) {
      this.assetService.release(url);
      throw error;
    }
  }
}
