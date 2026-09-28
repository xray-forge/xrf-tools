import { jest } from "@jest/globals";

import { TransportEndpoint } from "@/core/ipc/types/xrf-app";

export type BulkHandler = (args: Record<string, unknown>) => unknown;

/** A handler answering bytes, for a test driving a route with a `jest.fn`. */
export type BulkRead = (args: Record<string, unknown>) => Promise<ArrayBuffer>;

/**
 * Answers keyed by bulk route, such as `levels/read_sector`: bytes, or a handler answering them.
 *
 * A handler that throws or rejects is the route failing with that message, as the backend answers a failed command.
 */
export type BulkMap = Record<string, unknown | BulkHandler>;

/** Where the mocked transport listens, which `transport|get_endpoint` answers unless a test says otherwise. */
export const MOCK_TRANSPORT_ENDPOINT: TransportEndpoint = { origin: "http://127.0.0.1:1", token: "token" };

const state: { handlers: BulkMap } = { handlers: {} };

/**
 * Configures what the mocked transport answers.
 *
 * @param handlers - Answers or handlers keyed by route.
 */
export function setMockBulkResponses(handlers: BulkMap): void {
  state.handlers = handlers;
}

/** Clears every mocked transport answer. */
export function resetMockBulk(): void {
  state.handlers = {};
  mockFetch.mockClear();
}

/**
 * @param url - A bulk request's url.
 * @returns The route it names.
 */
function toMockBulkRoute(url: string): string {
  return url.startsWith(MOCK_TRANSPORT_ENDPOINT.origin) ? url.slice(MOCK_TRANSPORT_ENDPOINT.origin.length + 1) : url;
}

/**
 * @param route - A bulk route, such as `assets/read_asset`.
 * @returns The arguments of every fetch of it since the last reset, in the order they were made.
 */
export function listMockBulkCalls(route: string): Array<Record<string, unknown>> {
  return mockFetch.mock.calls
    .filter(([input]) => toMockBulkRoute(String(input)) === route)
    .map(([, init]) => JSON.parse(String(init?.body ?? "{}")) as Record<string, unknown>);
}

/**
 * @param bytes - What a route answered.
 * @returns It as the buffer a response body reads as: a view's own region, never the buffer around it.
 */
function toBuffer(bytes: unknown): ArrayBuffer {
  if (bytes instanceof ArrayBuffer) {
    return bytes;
  }

  if (ArrayBuffer.isView(bytes)) {
    return bytes.buffer.slice(bytes.byteOffset, bytes.byteOffset + bytes.byteLength) as ArrayBuffer;
  }

  throw new Error(`A mocked bulk route answered ${typeof bytes}, not bytes`);
}

/** A response as much as the transport's callers read of one. */
function toResponse(status: number, body: ArrayBuffer | string): Response {
  const isText: boolean = typeof body === "string";

  return {
    arrayBuffer: async () => (isText ? new TextEncoder().encode(body as string).buffer : body),
    headers: new Map([["content-type", isText ? "application/json" : "application/octet-stream"]]),
    ok: status >= 200 && status < 300,
    status,
    statusText: "",
    text: async () => (isText ? (body as string) : ""),
  } as unknown as Response;
}

/**
 * The page's `fetch`, answering a bulk route from the configured map; a route nobody configured fails as the backend
 * does for a command it does not know.
 */
export const mockFetch = jest.fn(async (input: string | URL | Request, init?: RequestInit): Promise<Response> => {
  const route: string = toMockBulkRoute(String(input));
  const handler: unknown = state.handlers[route];

  if (handler === undefined) {
    return toResponse(404, JSON.stringify(`No mocked answer for transport route '${route}'`));
  }

  try {
    const args: Record<string, unknown> = init?.body ? JSON.parse(String(init.body)) : {};
    const answer: unknown = typeof handler === "function" ? await (handler as BulkHandler)(args) : handler;

    return toResponse(200, toBuffer(answer));
  } catch (error: unknown) {
    return toResponse(500, JSON.stringify(error instanceof Error ? error.message : String(error)));
  }
});
