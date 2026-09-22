import { Injectable, OnDeprovision, OnProvision, ProvisionId } from "@wirestate/core";
import { BoundAction, Observable } from "@wirestate/mobx";
import { ERenderResolution, TFrameRateLimit, toFrameRateLimit, toRenderResolution } from "@xrf/renderer";
import { Nullable } from "@xrf/types";

import { TCatalogView, toCatalogView } from "@/core/settings/lib/catalog-view";
import {
  CATALOG_VIEW_STORAGE_KEY,
  DEV_MODE_STORAGE_KEY,
  FRAME_RATE_LIMIT_STORAGE_KEY,
  OFFSCREEN_RENDER_STORAGE_KEY,
  RENDER_RESOLUTION_STORAGE_KEY,
} from "@/core/storage";
import { isDevelopmentBuild } from "@/lib/env";
import { getLocalStorageValue, setLocalStorageValue } from "@/lib/local-storage";
import { Logger } from "@/lib/logging";

/**
 * Application wide switches that are not tied to any one editor.
 */
@Injectable()
export class SettingsService {
  public readonly log: Logger = new Logger(__MODULE_NAME__);

  /** Surfaces dev traces and captured runtime errors that are otherwise hidden. */
  @Observable()
  public isDevModeEnabled: boolean = SettingsService.readDevModeEnabled();

  /** How the root catalog draws its tools. */
  @Observable()
  public catalogView: TCatalogView = toCatalogView(getLocalStorageValue(CATALOG_VIEW_STORAGE_KEY));

  /** Whether a level draws on a thread of its own, which it does unless somebody has said not to. */
  @Observable()
  public isOffscreenRenderEnabled: boolean = getLocalStorageValue(OFFSCREEN_RENDER_STORAGE_KEY) !== String(false);

  /**
   * Frames a second every viewport is allowed to draw.
   */
  @Observable()
  public frameRateLimit: TFrameRateLimit = toFrameRateLimit(getLocalStorageValue(FRAME_RATE_LIMIT_STORAGE_KEY));

  @Observable()
  public renderResolution: ERenderResolution = toRenderResolution(getLocalStorageValue(RENDER_RESOLUTION_STORAGE_KEY));

  /**
   * @returns The stored choice, or whether this is a development build when there is none.
   */
  private static readDevModeEnabled(): boolean {
    const stored: Nullable<string> = getLocalStorageValue(DEV_MODE_STORAGE_KEY);

    return stored === null ? isDevelopmentBuild() : stored === String(true);
  }

  @OnProvision()
  public async onProvision(provisionId: ProvisionId): Promise<void> {
    this.log.info("Provisioning:", provisionId);
  }

  @OnDeprovision()
  public onDeprovision(provisionId: ProvisionId): void {
    this.log.info("Deprovisioning:", provisionId);
  }

  @BoundAction()
  public setDevModeEnabled(isEnabled: boolean): void {
    this.log.info("Set dev mode:", isEnabled);

    this.isDevModeEnabled = isEnabled;
    setLocalStorageValue(DEV_MODE_STORAGE_KEY, String(isEnabled));
  }

  @BoundAction()
  public setOffscreenRenderEnabled(isEnabled: boolean): void {
    this.log.info("Set offscreen render:", isEnabled);

    this.isOffscreenRenderEnabled = isEnabled;
    setLocalStorageValue(OFFSCREEN_RENDER_STORAGE_KEY, String(isEnabled));
  }

  @BoundAction()
  public setFrameRateLimit(limit: TFrameRateLimit): void {
    this.log.info("Set frame rate limit:", limit);

    this.frameRateLimit = limit;
    setLocalStorageValue(FRAME_RATE_LIMIT_STORAGE_KEY, limit);
  }

  @BoundAction()
  public setRenderResolution(resolution: ERenderResolution): void {
    this.log.info("Set render resolution:", resolution);

    this.renderResolution = resolution;
    setLocalStorageValue(RENDER_RESOLUTION_STORAGE_KEY, resolution);
  }

  @BoundAction()
  public setCatalogView(view: TCatalogView): void {
    this.log.info("Set catalog view:", view);

    this.catalogView = view;
    setLocalStorageValue(CATALOG_VIEW_STORAGE_KEY, view);
  }
}
