import { Maybe } from "@xrf/types";

import { IRendererFetchBatch } from "#/contract/scene/renderer-fetch-batch";
import { IFetchPart } from "#/texture/fetch-part";
import { readFetchFailure } from "#/texture/read-fetch-failure";
import { readFetchParts } from "#/texture/read-fetch-parts";
import { IRendererFetchedBytes } from "#/texture/renderer-fetched-bytes";

/** Most calls one batch carries, well under what the server takes. */
const MAXIMUM_CALLS: number = 64;
/**
 * Batches posted at once. While they are all out, what is asked for waits and goes as the next batch, so batches grow
 * as fast as requests come; the browser keeps six connections to a server, and the level's other fetches need theirs.
 */
const MAXIMUM_IN_FLIGHT: number = 4;

/** A request waiting for its batch's part. */
interface IBatchedFetch {
  batch: IRendererFetchBatch;
  headers: Record<string, string>;
  /** Its headers as one key, which requests going together share. */
  group: string;
  signal: AbortSignal;
  resolve: (fetched: IRendererFetchedBytes) => void;
  reject: (error: unknown) => void;
  onAbort: () => void;
  /** Whether it was answered, refused or aborted, so nothing settles it twice. */
  isSettled: boolean;
  /** Aborts the batch it went in, once every call of it was aborted; null while it waits. */
  sent: Maybe<IBatchInFlight>;
}

/** A batch posted and not yet read to its end. */
interface IBatchInFlight {
  controller: AbortController;
  calls: Array<IBatchedFetch>;
}

/**
 * Sends requests that may go together as batches, since a fetch costs the worker about 0.85 ms whatever it carries. A
 * request waits only while every batch allowed is out, and is answered as its part comes in, not with its batch.
 */
export class RendererFetchBatches {
  private readonly waiting: Array<IBatchedFetch> = [];
  private readonly post: (url: string, init: RequestInit) => Promise<Response>;
  private inFlight: number = 0;
  private isScheduled: boolean = false;

  /**
   * @param post - Makes a request; the global `fetch` as it stands when called by default.
   */
  public constructor(post: (url: string, init: RequestInit) => Promise<Response> = (url, init) => fetch(url, init)) {
    this.post = post;
  }

  /**
   * @param batch - Where the request goes together with others, and as what call.
   * @param headers - Its headers, which every request of its batch carries.
   * @param signal - Aborts it.
   * @returns Its bytes and their media type; refused with the server's own message where it answered one.
   */
  public fetch(
    batch: IRendererFetchBatch,
    headers: Record<string, string>,
    signal: AbortSignal
  ): Promise<IRendererFetchedBytes> {
    if (signal.aborted) {
      return Promise.reject(signal.reason);
    }

    return new Promise((resolve, reject) => {
      const call: IBatchedFetch = {
        batch,
        group: `${batch.url}\n${JSON.stringify(headers)}`,
        headers,
        isSettled: false,
        onAbort: () => this.abort(call),
        reject,
        resolve,
        sent: null,
        signal,
      };

      signal.addEventListener("abort", call.onAbort, { once: true });
      this.waiting.push(call);
      this.schedule();
    });
  }

  /** Sends what waits once the requests made now are all in, rather than one batch each. */
  private schedule(): void {
    if (!this.isScheduled) {
      this.isScheduled = true;
      queueMicrotask(() => {
        this.isScheduled = false;
        this.pump();
      });
    }
  }

  /** Posts batches of what waits, oldest first and each of one group, while fewer than allowed are out. */
  private pump(): void {
    while (this.inFlight < MAXIMUM_IN_FLIGHT && this.waiting.length) {
      const group: string = this.waiting[0].group;
      const calls: Array<IBatchedFetch> = [];

      for (let at: number = 0; at < this.waiting.length && calls.length < MAXIMUM_CALLS;) {
        if (this.waiting[at].group === group) {
          calls.push(...this.waiting.splice(at, 1));
        } else {
          at += 1;
        }
      }

      void this.send({ calls, controller: new AbortController() });
    }
  }

  private async send(batch: IBatchInFlight): Promise<void> {
    const { calls, controller } = batch;
    const [{ batch: first, headers }] = calls;

    this.inFlight += 1;
    calls.forEach((call: IBatchedFetch) => (call.sent = batch));

    try {
      const response: Response = await this.post(first.url, {
        body: `[${calls.map((call: IBatchedFetch) => call.batch.call).join(",")}]`,
        headers,
        method: "POST",
        signal: controller.signal,
      });

      if (!response.ok) {
        throw new Error(readFetchFailure(response.status, await response.text()));
      }

      if (!response.body) {
        throw new Error("The batch was answered without a body");
      }

      await readFetchParts(response.body, (part: IFetchPart) => this.answer(calls[part.index], part));

      // Read to its end with a call unanswered: a server that dropped one would leave it waiting for good.
      calls.forEach((call: IBatchedFetch) => this.refuse(call, new Error("The batch ended without answering it")));
    } catch (error: unknown) {
      // A broken or refused answer ends the request too, so nothing more of it is read or sent.
      controller.abort(error);
      calls.forEach((call: IBatchedFetch) => this.refuse(call, error));
    } finally {
      this.inFlight -= 1;
      this.pump();
    }
  }

  private answer(call: Maybe<IBatchedFetch>, part: IFetchPart): void {
    if (!call || !this.settle(call)) {
      return;
    }

    if (part.status >= 200 && part.status < 300) {
      call.resolve({ bytes: part.bytes, type: part.type || null });
    } else {
      call.reject(new Error(readFetchFailure(part.status, new TextDecoder().decode(part.bytes))));
    }
  }

  private refuse(call: IBatchedFetch, error: unknown): void {
    if (this.settle(call)) {
      call.reject(error);
    }
  }

  /** Refuses an aborted request, and aborts its batch once nothing of it is waited for any more. */
  private abort(call: IBatchedFetch): void {
    const index: number = this.waiting.indexOf(call);

    if (index >= 0) {
      this.waiting.splice(index, 1);
    }

    this.refuse(call, call.signal.reason);

    if (call.sent?.calls.every((it: IBatchedFetch) => it.isSettled)) {
      call.sent.controller.abort(call.signal.reason);
    }
  }

  /** @returns Whether the request was still open, now settled. */
  private settle(call: IBatchedFetch): boolean {
    if (call.isSettled) {
      return false;
    }

    call.isSettled = true;
    call.signal.removeEventListener("abort", call.onAbort);

    return true;
  }
}
