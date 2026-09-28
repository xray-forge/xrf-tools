import { Nullable } from "@xrf/types";

import { transportCommands } from "@/core/ipc/commands/transport";
import { TransportEndpoint } from "@/core/ipc/types/xrf-app";

/**
 * Where the transport listens, asked of the backend once and kept for as long as the page lives: the port and token
 * are the launch's, and a reload asks again.
 */
export class BulkEndpoint {
  private readonly ask: () => Promise<TransportEndpoint>;
  private endpoint: Nullable<Promise<TransportEndpoint>> = null;

  /**
   * @param ask - Asks the backend where the transport listens.
   */
  public constructor(ask: () => Promise<TransportEndpoint>) {
    this.ask = ask;
  }

  /**
   * @returns Where the transport listens; an ask that failed is forgotten, so the next call asks again.
   */
  public get(): Promise<TransportEndpoint> {
    if (this.endpoint) {
      return this.endpoint;
    }

    const asked: Promise<TransportEndpoint> = this.ask();

    this.endpoint = asked;
    asked.catch((): void => {
      if (this.endpoint === asked) {
        this.endpoint = null;
      }
    });

    return asked;
  }
}

/** The endpoint every bulk call is made against. */
export const BULK_ENDPOINT: BulkEndpoint = new BulkEndpoint(() => transportCommands.getEndpoint());
