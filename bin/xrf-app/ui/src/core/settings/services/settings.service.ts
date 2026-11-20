import { Injectable, OnDeprovision, OnProvision, ProvisionId } from "@wirestate/core";
import { BoundAction, Computed } from "@wirestate/mobx";
import { Logger } from "@xrf/logging";

import { TCatalogView, toCatalogView } from "@/core/settings/lib/catalog-view";
import { booleanCodec, CATALOG_VIEW_STORAGE_KEY, DEV_MODE_STORAGE_KEY, StoredValue, textCodec } from "@/core/storage";
import { isDevelopmentBuild } from "@/lib/env/build-mode";

/**
 * Application wide switches that are not tied to any one editor.
 */
@Injectable()
export class SettingsService {
  public readonly log: Logger = new Logger(__MODULE_NAME__);

  /** The stored choice, or whether this is a development build when there is none. */
  private readonly storedDevModeEnabled: StoredValue<boolean> = new StoredValue(
    DEV_MODE_STORAGE_KEY,
    booleanCodec(isDevelopmentBuild())
  );

  private readonly storedCatalogView: StoredValue<TCatalogView> = new StoredValue(
    CATALOG_VIEW_STORAGE_KEY,
    textCodec(toCatalogView)
  );

  /** Surfaces dev traces and captured runtime errors that are otherwise hidden. */
  @Computed()
  public get isDevModeEnabled(): boolean {
    return this.storedDevModeEnabled.value;
  }

  /** How the root catalog draws its tools. */
  @Computed()
  public get catalogView(): TCatalogView {
    return this.storedCatalogView.value;
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

    this.storedDevModeEnabled.set(isEnabled);
  }

  @BoundAction()
  public setCatalogView(view: TCatalogView): void {
    this.log.info("Set catalog view:", view);

    this.storedCatalogView.set(view);
  }
}
