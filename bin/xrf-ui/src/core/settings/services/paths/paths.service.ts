import { Injectable, OnDeprovision, OnProvision, ProvisionId } from "@wirestate/core";
import { BoundAction, Observable } from "@wirestate/mobx";

import {
  createEmptyWorkspacePaths,
  EWorkspacePath,
  getWorkspacePath,
  IWorkspacePathDescriptor,
  TWorkspacePaths,
  WORKSPACE_PATHS,
} from "@/core/settings/lib/workspace-path";
import { getLocalStorageValue, setLocalStorageValue } from "@/lib/local-storage";
import { Logger } from "@/lib/logging";
import { Nullable, Optional } from "@/lib/types/general";

/**
 * The paths every tool derives its suggestions from.
 *
 * Read straight out of local storage as the service is constructed rather than restored asynchronously, because that
 * is all reading a remembered path costs and a tool mounting first would otherwise derive from nothing. A stored path
 * is kept exactly as the user left it: whether it still exists is reported where it is shown, never silently dropped.
 */
@Injectable()
export class PathsService {
  private static read(): TWorkspacePaths {
    const paths: TWorkspacePaths = createEmptyWorkspacePaths();

    for (const { id, storageKey } of WORKSPACE_PATHS) {
      paths[id] = getLocalStorageValue(storageKey);
    }

    return paths;
  }

  public readonly log: Logger = new Logger(__MODULE_NAME__);

  @Observable()
  public paths: TWorkspacePaths = PathsService.read();

  @OnProvision()
  public onProvision(provisionId: ProvisionId): void {
    this.log.info("Provisioning:", provisionId);
  }

  @OnDeprovision()
  public onDeprovision(provisionId: ProvisionId): void {
    this.log.info("Deprovisioning:", provisionId);
  }

  /**
   * @param id - Path to read.
   * @returns The configured path, or `null` when it is not set.
   */
  public getPath(id: EWorkspacePath): Nullable<string> {
    return this.paths[id];
  }

  /**
   * Replaces the whole record rather than one field, so a single observable covers every path.
   *
   * @param id - Path to write.
   * @param value - Path to remember, or `null` to forget it.
   */
  @BoundAction()
  public setPath(id: EWorkspacePath, value: Nullable<string>): void {
    const descriptor: Optional<IWorkspacePathDescriptor> = getWorkspacePath(id);

    if (!descriptor) {
      return this.log.error("Refusing to set a path nothing describes:", id);
    }

    this.log.info("Set path:", id, value);

    this.paths = { ...this.paths, [id]: value };
    setLocalStorageValue(descriptor.storageKey, value);
  }
}
