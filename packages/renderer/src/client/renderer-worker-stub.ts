import { Nullable } from "@xrf/types";

import { ERendererRequest, TRendererRequest, TRendererResponse } from "#/contract/renderer-messages";

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
  /**
   * @param kind - Which requests to keep.
   * @returns Those requests, in order.
   */
  take<K extends ERendererRequest>(kind: K): Array<Extract<TRendererRequest, { kind: K }>>;
  /**
   * @param response - What the renderer says back.
   */
  respond(response: TRendererResponse): void;
}

/**
 * @returns A worker stub with nothing posted yet.
 */
export function createRendererWorkerStub(): IRendererWorkerStub {
  const requests: Array<TRendererRequest> = [];
  let isTerminated: boolean = false;

  const worker = {
    onerror: null,
    onmessage: null as Nullable<(event: MessageEvent<TRendererResponse>) => void>,
    postMessage: (request: TRendererRequest): void => {
      requests.push(request);
    },
    terminate: (): void => {
      isTerminated = true;
    },
  };

  return {
    isTerminated: () => isTerminated,
    requests,
    respond: (response: TRendererResponse): void => worker.onmessage?.({ data: response } as MessageEvent),
    take: <K extends ERendererRequest>(kind: K) =>
      requests.filter((request): request is Extract<TRendererRequest, { kind: K }> => request.kind === kind),
    worker: worker as unknown as Worker,
  };
}
