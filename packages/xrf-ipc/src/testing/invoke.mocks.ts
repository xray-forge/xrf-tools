import { jest } from "@jest/globals";
import { Optional } from "@xrf/types";

// Types only: a command module's runtime imports the real `invoke`, which would bind Tauri's own `invoke` before a test
// setup's `jest.mock` of it takes effect.
import type { IIpcCommandSignatures } from "#/commands/signatures";
import type { ETransportCommand } from "#/commands/transport";
import { MOCK_TRANSPORT_ENDPOINT } from "#/testing/bulk.mocks";

/** Answers one command of `K` with what it resolves to, given the arguments it was invoked with. */
export type IpcCommandAnswer<K extends keyof IIpcCommandSignatures> = (
  args: IIpcCommandSignatures[K]["args"]
) => IIpcCommandSignatures[K]["result"] | Promise<IIpcCommandSignatures[K]["result"]>;

/**
 * What a test answers one command with: what it resolves to, or an answer given the arguments it was invoked with.
 *
 * An answer that throws or rejects is the command failing with that error, as the backend rejects one.
 */
export type IpcCommandMock<K extends keyof IIpcCommandSignatures> =
  IIpcCommandSignatures[K]["result"] | IpcCommandAnswer<K>;

/**
 * Answers keyed by command member, each typed by what that command takes and resolves to.
 *
 * Anything not listed resolves to `null` rather than throwing, so a test only has to describe the commands it actually
 * cares about.
 */
export type IpcCommandMocks = { [K in keyof IIpcCommandSignatures]?: IpcCommandMock<K> };

/** Whether tests are running in the mocked desktop runtime. */
export const mockIsTauri = jest.fn((): boolean => true);

const state: { handlers: Partial<Record<string, unknown>> } = { handlers: {} };

/** The transport's endpoint command, spelled out since its enum cannot be loaded here, and checked against it. */
const GET_ENDPOINT: `${ETransportCommand.GET_ENDPOINT}` = "plugin:transport|get_endpoint";

/** What a command answers when a test configures nothing for it, beside the null everything else answers. */
const DEFAULT_RESPONSES: Partial<Record<string, unknown>> = { [GET_ENDPOINT]: MOCK_TRANSPORT_ENDPOINT };

/**
 * Configures what mocked commands answer.
 *
 * @param mocks - Answers keyed by command member, such as `EVisualsCommand.OPEN_MODEL`.
 */
export function mockIpcCommands(mocks: IpcCommandMocks): void {
  state.handlers = mocks;
}

/**
 * Clears all mocked Tauri command responses.
 *
 */
export function resetMockInvoke(): void {
  state.handlers = {};
}

/** Restores the mocked runtime to the desktop default. */
export function resetMockIsTauri(): void {
  mockIsTauri.mockReset();
  mockIsTauri.mockReturnValue(true);
}

/**
 * Invokes a configured mocked Tauri command.
 *
 * @param command - Command name to invoke.
 * @param args - Arguments passed to the configured handler.
 * @returns A promise resolving to the configured response.
 */
export const mockInvoke = jest.fn(async (command: string, args?: Record<string, unknown>): Promise<unknown> => {
  const handler: unknown = state.handlers[command] ?? DEFAULT_RESPONSES[command];

  if (typeof handler === "function") {
    return (handler as (args?: Record<string, unknown>) => unknown)(args);
  }

  return handler ?? null;
});

/**
 * @param command - A command member, such as `ERenderCommand.SET_CAMERA`.
 * @returns The arguments of every invoke of it since the last reset, in the order they were made.
 */
export function listMockInvokeCalls<K extends keyof IIpcCommandSignatures>(
  command: K
): Array<IIpcCommandSignatures[K]["args"]> {
  return mockInvoke.mock.calls
    .filter(([name]) => name === command)
    .map(([, args]) => args as IIpcCommandSignatures[K]["args"]);
}

/**
 * Channels constructed since the last reset, in creation order.
 *
 * A test drives progress by taking the channel a command was handed and calling its `onmessage`, which is exactly what
 * the backend does through the real transport.
 */
const channelState: { created: Array<MockChannel<unknown>> } = { created: [] };

/**
 * Stand-in for the Tauri IPC channel.
 *
 * Carries the same two members production code touches - an id assigned on construction and a settable `onmessage` -
 * so a service that builds one, hands it over, and routes what arrives behaves here as it does in the desktop app.
 */
export class MockChannel<T> {
  public static nextId: number = 0;

  public id: number;
  public onmessage: (message: T) => void = () => {};

  public constructor(onmessage?: (message: T) => void) {
    this.id = ++MockChannel.nextId;

    if (onmessage) {
      this.onmessage = onmessage;
    }

    channelState.created.push(this as MockChannel<unknown>);
  }
}

/**
 * @returns Channels constructed since the last reset, in creation order.
 */
export function getMockChannels(): Array<MockChannel<unknown>> {
  return channelState.created;
}

/**
 * Delivers a message through the most recently constructed channel.
 *
 * @param message - Payload to deliver.
 */
export function emitMockChannelMessage(message: unknown): void {
  const channel: Optional<MockChannel<unknown>> = channelState.created.at(-1);

  if (!channel) {
    throw new Error("No channel has been constructed to deliver a message through.");
  }

  channel.onmessage(message);
}

/** Forgets every channel a test constructed. */
export function resetMockChannels(): void {
  channelState.created = [];
}
