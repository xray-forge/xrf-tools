import { exists } from "@tauri-apps/plugin-fs";
import { Injectable, OnDeprovision, OnProvision, ProvisionId } from "@wirestate/core";
import { BoundAction, flowResult, Observable } from "@wirestate/mobx";

import { getLocalStorageValue, setLocalStorageValue } from "@/lib/local-storage";
import { Logger } from "@/lib/logging";
import { call, ExclusiveFlow, TFlow } from "@/lib/mobx";
import { Nullable } from "@/lib/types/general";

@Injectable()
export class ProjectService {
  public readonly log: Logger = new Logger(__MODULE_NAME__);

  @Observable()
  public xrfProjectPath: Nullable<string> = null;

  @OnProvision()
  public async onProvision(provisionId: ProvisionId): Promise<void> {
    this.log.info("Provisioning:", provisionId);

    await flowResult(this.restore());
  }

  @OnDeprovision()
  public onDeprovision(provisionId: ProvisionId): void {
    this.log.info("Deprovisioning:", provisionId);
  }

  /**
   * Reads back the path this project was last pointed at.
   *
   * Exclusive so a second provisioning joins the read in flight rather than issuing another, and so a path the user
   * sets meanwhile is not overwritten by an answer that was already on its way.
   */
  @ExclusiveFlow("xrfProjectPath")
  private *restore(): TFlow {
    const path: Nullable<string> = yield* call(this.getXrfProjectPath());

    this.log.info("Loaded getXrfProjectPath:", path);

    this.xrfProjectPath = path;
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
