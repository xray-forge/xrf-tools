import { Injectable, OnDeprovision, OnProvision, ProvisionId } from "@wirestate/core";
import { BoundAction, Observable } from "@wirestate/mobx";

import { TCatalogView, toCatalogView } from "@/core/settings/lib/catalog-view";
import { isDevelopmentBuild } from "@/lib/env";
import { getLocalStorageValue, setLocalStorageValue } from "@/lib/local-storage";
import { Logger } from "@/lib/logging";
import { Nullable } from "@/lib/types/general";

/**
 * Application wide switches that are not tied to any one editor.
 */
@Injectable()
export class SettingsService {
  private static readonly DEV_MODE_STORAGE_KEY: string = "xrf-dev-mode";
  private static readonly CATALOG_VIEW_STORAGE_KEY: string = "xrf-catalog-view";

  public readonly log: Logger = new Logger(__MODULE_NAME__);

  /** Surfaces dev traces and captured runtime errors that are otherwise hidden. */
  @Observable()
  public isDevModeEnabled: boolean = SettingsService.readDevModeEnabled();

  /** How the root catalog draws its tools. */
  @Observable()
  public catalogView: TCatalogView = toCatalogView(getLocalStorageValue(SettingsService.CATALOG_VIEW_STORAGE_KEY));

  /**
   * @returns The stored choice, or whether this is a development build when there is none.
   */
  private static readDevModeEnabled(): boolean {
    const stored: Nullable<string> = getLocalStorageValue(SettingsService.DEV_MODE_STORAGE_KEY);

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
    setLocalStorageValue(SettingsService.DEV_MODE_STORAGE_KEY, String(isEnabled));
  }

  @BoundAction()
  public setCatalogView(view: TCatalogView): void {
    this.log.info("Set catalog view:", view);

    this.catalogView = view;
    setLocalStorageValue(SettingsService.CATALOG_VIEW_STORAGE_KEY, view);
  }
}
