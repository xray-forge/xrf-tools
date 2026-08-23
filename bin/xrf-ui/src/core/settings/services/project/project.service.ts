import { exists } from "@tauri-apps/plugin-fs";
import { Injectable, OnDeprovision, OnProvision, ProvisionId, WireStatus } from "@wirestate/core";
import { BoundAction, makeObservable, Observable, runInAction } from "@wirestate/mobx";

import { getLocalStorageValue, setLocalStorageValue } from "@/lib/local-storage";
import { Logger } from "@/lib/logging";
import { Nullable } from "@/lib/types/general";

@Injectable()
export class ProjectService {
  public readonly log: Logger = new Logger(__MODULE_NAME__);

  @Observable()
  public xrfProjectPath: Nullable<string> = null;

  public constructor(private readonly status: WireStatus = WireStatus.track(this)) {
    makeObservable(this);
  }

  @OnProvision()
  public onProvision(provisionId: ProvisionId): void {
    this.log.info("Provisioning:", provisionId);

    this.getXrfProjectPath().then((path) => {
      if (provisionId === this.status.provisionId) {
        this.log.info("Loaded getXrfProjectPath:", path);
        runInAction(() => (this.xrfProjectPath = path));
      }
    });
  }

  @OnDeprovision()
  public onDeprovision(provisionId: ProvisionId): void {
    this.log.info("Deprovisioning:", provisionId);
  }

  @BoundAction()
  public setXrfProjectPath(path: Nullable<string>): void {
    this.log.info("Set xrf project path:", path);

    this.xrfProjectPath = path;
    setLocalStorageValue("xrf-project-path", path);
  }

  public async getXrfProjectPath(): Promise<Nullable<string>> {
    const xrfProjectPath: Nullable<string> = getLocalStorageValue("xrf-project-path");

    if (xrfProjectPath && (await exists(xrfProjectPath))) {
      return xrfProjectPath;
    }

    return null;
  }
}
