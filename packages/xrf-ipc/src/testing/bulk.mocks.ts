import { jest } from "@jest/globals";

// Types only, as in `invoke.mocks`: no command module may load before a test setup mocks Tauri.
import type { IIpcBulkRouteSignatures } from "#/commands/signatures";
import type { TransportEndpoint } from "#/types/xrf-app-core";

/** What a route answers: bytes, as a buffer or a view of one. */
export type IpcBulkBytes = ArrayBuffer | ArrayBufferView;

/** Answers one route of `K` with bytes, given the arguments it was fetched with. */
export type IpcBulkRouteRead<K extends keyof IIpcBulkRouteSignatures> = (
  args: IIpcBulkRouteSignatures[K]["args"]
) => IpcBulkBytes | Promise<IpcBulkBytes>;

/** An asynchronous reader of one route of `K`, the shape a `jest.fn` driving it takes. */
export type IpcBulkRouteFetch<K extends keyof IIpcBulkRouteSignatures> = (
  args: IIpcBulkRouteSignatures[K]["args"]
) => Promise<ArrayBuffer>;

/**
 * What a test answers one bulk route with: bytes, or a reader given the arguments the route was fetched with.
 *
 * A reader that throws or rejects is the route failing with that message, as the backend answers a failed command.
 */
export type IpcBulkRouteMock<K extends keyof IIpcBulkRouteSignatures> = IpcBulkBytes | IpcBulkRouteRead<K>;

/** Answers keyed by bulk route member, each typed by what that route takes. */
export type IpcBulkRouteMocks = { [K in keyof IIpcBulkRouteSignatures]?: IpcBulkRouteMock<K> };

/** Where the mocked transport listens, which `transport|get_endpoint` answers unless a test says otherwise. */
export const MOCK_TRANSPORT_ENDPOINT: TransportEndpoint = { origin: "http://127.0.0.1:1", token: "token" };

const state: { handlers: Partial<Record<string, unknown>> } = { handlers: {} };

/**
 * Configures what the mocked transport answers.
 *
 * @param mocks - Answers or readers keyed by route member, such as `ELevelsBulkRoute.READ_SCREENSHOT`.
 */
export function mockIpcBulkRoutes(mocks: IpcBulkRouteMocks): void {
  state.handlers = mocks;
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
 * @param route - A bulk route member, such as `EAssetsBulkRoute.READ_ASSET`.
 * @returns The arguments of every fetch of it since the last reset, in the order they were made.
 */
export function listMockBulkCalls<K extends keyof IIpcBulkRouteSignatures>(
  route: K
): Array<IIpcBulkRouteSignatures[K]["args"]> {
  return mockFetch.mock.calls
    .filter(([input]) => toMockBulkRoute(String(input)) === route)
    .map(([, init]) => JSON.parse(String(init?.body ?? "{}")) as IIpcBulkRouteSignatures[K]["args"]);
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
    const answer: unknown =
      typeof handler === "function" ? await (handler as (args: Record<string, unknown>) => unknown)(args) : handler;

    return toResponse(200, toBuffer(answer));
  } catch (error: unknown) {
    return toResponse(500, JSON.stringify(error instanceof Error ? error.message : String(error)));
  }
});
