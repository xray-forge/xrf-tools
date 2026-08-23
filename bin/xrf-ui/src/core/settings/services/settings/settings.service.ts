import { Injectable, OnDeprovision, OnProvision, ProvisionId } from "@wirestate/core";
import { BoundAction, makeObservable, Observable } from "@wirestate/mobx";

import { isDevelopmentBuild } from "@/lib/env";
import { parseLocalStorageValue, setLocalStorageValue } from "@/lib/local-storage";
import { Logger } from "@/lib/logging";

/**
 * Application wide switches that are not tied to any one editor.
 */
@Injectable()
export class SettingsService {
  private static readonly DEV_MODE_STORAGE_KEY: string = "xrf-dev-mode";

  public readonly log: Logger = new Logger(__MODULE_NAME__);

  /** Surfaces dev traces and captured runtime errors that are otherwise hidden. */
  @Observable()
  public isDevModeEnabled: boolean =
    (parseLocalStorageValue(SettingsService.DEV_MODE_STORAGE_KEY) ?? isDevelopmentBuild()) === true;

  public constructor() {
    makeObservable(this);
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
}
