import { Nullable } from "@xrf/types";

import { ERendererRequest, TRendererRequest } from "#/contract/renderer-request";
import { TRendererResponse } from "#/contract/renderer-response";

/**
 * A stand-in for the renderer's worker, for tests of a consumer: it keeps what it was told and answers as told to.
 */
export interface IRendererWorkerStub {
  /** Handed to the client in place of `createRendererWorker()`. */
  worker: Worker;
  /** Every request posted, in order. */
  requests: Array<TRendererRequest>;
  /** Whether the client let the thread go. */
  isTerminated(): boolean;
  /** Waits out the page task, so what the client queued in it is posted. */
  flush(): Promise<void>;
  /**
   * @param kind - Which requests to keep.
   * @returns Those requests, in order.
   */
  take<K extends ERendererRequest>(kind: K): Array<Extract<TRendererRequest, { kind: K }>>;
  /**
   * @param response - What the renderer says back.
   */
  respond(response: TRendererResponse): void;
  /**
   * @param message - What the worker failed with, as an error thrown on its thread and never caught reports it.
   */
  crash(message: string): void;
}

/**
 * @returns A worker stub with nothing posted yet.
 */
export function createRendererWorkerStub(): IRendererWorkerStub {
  const requests: Array<TRendererRequest> = [];
  let isTerminated: boolean = false;

  const worker = {
    onerror: null as Nullable<(event: ErrorEvent) => void>,
    onmessage: null as Nullable<(event: MessageEvent<TRendererResponse>) => void>,
    // Batches are opened, so a test reads the requests the client made rather than how it grouped them.
    postMessage: (request: TRendererRequest): void => {
      requests.push(...(request.kind === ERendererRequest.BATCH ? request.requests : [request]));
    },
    terminate: (): void => {
      isTerminated = true;
    },
  };

  return {
    crash: (message: string): void => worker.onerror?.({ message } as ErrorEvent),
    flush: () => new Promise((resolve: () => void) => setTimeout(resolve, 0)),
    isTerminated: () => isTerminated,
    requests,
    respond: (response: TRendererResponse): void => worker.onmessage?.({ data: response } as MessageEvent),
    take: <K extends ERendererRequest>(kind: K) =>
      requests.filter(
        (request: TRendererRequest): request is Extract<TRendererRequest, { kind: K }> => request.kind === kind
      ),
    worker: worker as unknown as Worker,
  };
}
