import { Injectable, OnDeactivation } from "@wirestate/core";

import { Logger } from "@/lib/logging";
import { Nullable } from "@/lib/types/general";

/**
 * Owns the lifetime of every object url an editor hands to the webview.
 */
@Injectable()
export class AssetService {
  public readonly log: Logger = new Logger(__MODULE_NAME__);

  /** Urls held against a caller supplied key, so a new one can displace the old automatically. */
  private readonly keyed: Map<string, string> = new Map();

  /** Urls the caller owns outright and must release itself. */
  private readonly loose: Set<string> = new Set();

  /**
   * Sweep whatever the editor still held on the way out.
   */
  @OnDeactivation()
  public onDeactivation(): void {
    if (this.heldCount) {
      this.log.info("Releasing held object URLs:", this.heldCount);
    }

    this.releaseAll();
  }

  /**
   * Create a url the caller is responsible for releasing.
   *
   * Prefer `swap` where there is a natural key - a selection, a sprite - so releasing is not something
   * anyone has to remember.
   *
   * @param blob - Data for the object URL.
   * @returns An object URL the caller must release.
   */
  public create(blob: Blob): string {
    const url: string = URL.createObjectURL(blob);

    this.log.info("Create object URLs:", url);

    this.loose.add(url);

    return url;
  }

  /**
   * Replace whatever url is held under `key` and return the new one.
   *
   * The new url is created before the old is revoked, which is the ordering that matters: revoking
   * first leaves anything still rendering the old url pointing at nothing for as long as the
   * replacement takes, and permanently if creating it fails.
   *
   * @param key - Key whose current object URL is replaced.
   * @param blob - Data for the replacement object URL.
   * @returns The replacement object URL held under the key.
   */
  public swap(key: string, blob: Blob): string {
    const previous: Nullable<string> = this.keyed.get(key) ?? null;
    const url: string = URL.createObjectURL(blob);

    this.log.info("Swap object URLs:", url, "previous:", previous);

    this.keyed.set(key, url);

    if (previous) {
      URL.revokeObjectURL(previous);
    }

    return url;
  }

  /**
   * Releases one URL obtained from `create`.
   *
   * Unknown URLs are ignored rather than revoked blindly.
   *
   * @param url - Caller-owned object URL to release.
   */
  public release(url: Nullable<string>): void {
    if (url && this.loose.delete(url)) {
      this.log.info("Revoke object URL:", url);

      URL.revokeObjectURL(url);
    }
  }

  /**
   * Releases the object URL held under a key.
   *
   * @param key - Key whose object URL should be released.
   */
  public releaseKey(key: string): void {
    const url: Nullable<string> = this.keyed.get(key) ?? null;

    if (url) {
      this.log.info("Revoke object URL of key:", key, url);

      this.keyed.delete(key);
      URL.revokeObjectURL(url);
    }
  }

  /**
   * Counts the object URLs still held by the service.
   *
   * @returns The number of keyed and caller-owned object URLs.
   */
  public get heldCount(): number {
    return this.keyed.size + this.loose.size;
  }

  public releaseAll(): void {
    for (const url of this.keyed.values()) {
      this.log.info("Revoke object URL:", url);
      URL.revokeObjectURL(url);
    }

    for (const url of this.loose) {
      this.log.info("Revoke object URL:", url);
      URL.revokeObjectURL(url);
    }

    this.keyed.clear();
    this.loose.clear();
  }
}
