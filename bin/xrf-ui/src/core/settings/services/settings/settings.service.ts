import { Injectable, OnDeprovision, OnProvision, ProvisionId } from "@wirestate/core";
import { BoundAction, Observable, RefObservable } from "@wirestate/mobx";
import {
  ERendererPreset,
  ERenderResolution,
  IRendererFeatureChoice,
  IRendererFeatureOverrides,
  IRendererFeatureSettings,
  resolveRendererFeatures,
  TFrameRateLimit,
  toFrameRateLimit,
  toRendererFeatureChoice,
  toRenderResolution,
} from "@xrf/renderer";
import { Nullable } from "@xrf/types";

import { TCatalogView, toCatalogView } from "@/core/settings/lib/catalog-view";
import {
  CATALOG_VIEW_STORAGE_KEY,
  DEV_MODE_STORAGE_KEY,
  FRAME_RATE_LIMIT_STORAGE_KEY,
  RENDER_RESOLUTION_STORAGE_KEY,
  RENDERER_FEATURES_STORAGE_KEY,
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

  /**
   * Frames a second every viewport is allowed to draw.
   */
  @Observable()
  public frameRateLimit: TFrameRateLimit = toFrameRateLimit(getLocalStorageValue(FRAME_RATE_LIMIT_STORAGE_KEY));

  @Observable()
  public renderResolution: ERenderResolution = toRenderResolution(getLocalStorageValue(RENDER_RESOLUTION_STORAGE_KEY));

  /**
   * The renderer's preset and what was changed on top of it, the same for every viewport. Held by reference, so what
   * it resolves to crosses to the renderer's thread as plain data.
   */
  @RefObservable()
  public rendererChoice: IRendererFeatureChoice = SettingsService.readRendererChoice();

  /** Every renderer feature as the choice sets it. */
  public get rendererFeatures(): IRendererFeatureSettings {
    return resolveRendererFeatures(this.rendererChoice);
  }

  /**
   * @returns The stored choice, or the default where none was stored or it does not parse.
   */
  private static readRendererChoice(): IRendererFeatureChoice {
    try {
      return toRendererFeatureChoice(JSON.parse(getLocalStorageValue(RENDERER_FEATURES_STORAGE_KEY) ?? "null"));
    } catch {
      return toRendererFeatureChoice(null);
    }
  }

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

  /**
   * @param preset - The preset every feature follows from now on, whatever was changed on top of the last one.
   */
  @BoundAction()
  public setRendererPreset(preset: ERendererPreset): void {
    this.log.info("Set renderer preset:", preset);

    this.storeRendererChoice({ overrides: {}, preset });
  }

  /**
   * @param overrides - What changes on top of the preset, merged over what already did.
   */
  @BoundAction()
  public setRendererOverrides(overrides: IRendererFeatureOverrides): void {
    const current: IRendererFeatureOverrides = this.rendererChoice.overrides;

    this.storeRendererChoice({
      overrides: {
        ...current,
        ...overrides,
        ambientOcclusion:
          overrides.ambientOcclusion || current.ambientOcclusion
            ? { ...current.ambientOcclusion, ...overrides.ambientOcclusion }
            : undefined,
        grass: overrides.grass || current.grass ? { ...current.grass, ...overrides.grass } : undefined,
        lod: overrides.lod || current.lod ? { ...current.lod, ...overrides.lod } : undefined,
        shadows: overrides.shadows || current.shadows ? { ...current.shadows, ...overrides.shadows } : undefined,
      },
      preset: this.rendererChoice.preset,
    });
  }

  @BoundAction()
  public setCatalogView(view: TCatalogView): void {
    this.log.info("Set catalog view:", view);

    this.catalogView = view;
    setLocalStorageValue(CATALOG_VIEW_STORAGE_KEY, view);
  }

  private storeRendererChoice(choice: IRendererFeatureChoice): void {
    this.rendererChoice = toRendererFeatureChoice(choice);
    setLocalStorageValue(RENDERER_FEATURES_STORAGE_KEY, JSON.stringify(this.rendererChoice));
  }
}
