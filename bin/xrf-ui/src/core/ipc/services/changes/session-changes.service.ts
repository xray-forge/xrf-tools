import { Channel, isTauri } from "@tauri-apps/api/core";
import { EventBus, inject, Injectable } from "@wirestate/core";
import { transformError } from "@xrf/errors";
import { Logger } from "@xrf/logging";
import { Nullable } from "@xrf/types";

import { assetsCommands } from "@/core/ipc/commands/assets";
import { SESSION_CHANGE_EVENT } from "@/core/ipc/session";
import { AssetChangeEvent, SessionChange } from "@/core/ipc/types/xrf-app-core";

/**
 * The window's one subscription to committed changes: each change's sessions go out one at a time as
 * `SESSION_CHANGE_EVENT`, which the service holding that session's snapshot acts on and every other ignores.
 */
@Injectable()
export class SessionChangesService {
  public readonly log: Logger = new Logger(__MODULE_NAME__);

  /** The subscription once asked for, which the backend keeps until the page goes. */
  private subscription: Nullable<Promise<void>> = null;

  public constructor(private readonly eventBus: EventBus = inject(EventBus)) {}

  /**
   * Subscribes the window, once however often it is asked; one that failed is asked for again by the next call.
   *
   * @returns Resolves once the backend holds the subscription, or once it refused it.
   */
  public listen(): Promise<void> {
    if (!isTauri()) {
      return Promise.resolve();
    }

    this.subscription ??= this.subscribe();

    return this.subscription;
  }

  private async subscribe(): Promise<void> {
    const changes: Channel<AssetChangeEvent> = new Channel<AssetChangeEvent>();

    changes.onmessage = (event: AssetChangeEvent): void => this.route(event);

    try {
      await assetsCommands.subscribeChanges(changes);
    } catch (error: unknown) {
      this.subscription = null;
      this.log.error("Could not subscribe to committed changes:", transformError(error));
    }
  }

  private route(event: AssetChangeEvent): void {
    this.log.info("Change committed:", event.revision, "sessions:", event.sessions.length);

    for (const change of event.sessions) {
      this.eventBus.emit<SessionChange>(SESSION_CHANGE_EVENT, change);
    }
  }
}
