import { afterEach, jest } from "@jest/globals";
import { Logger } from "@xrf/logging";

import {
  MockChannel,
  mockFetch,
  mockInvoke,
  mockIsTauri,
  resetMockBulk,
  resetMockChannels,
  resetMockInvoke,
  resetMockIsTauri,
} from "#/testing/index";

jest.mock("@tauri-apps/api/core", () => ({
  Channel: MockChannel,
  invoke: mockInvoke,
  isTauri: mockIsTauri,
}));

Object.defineProperty(globalThis, "fetch", { configurable: true, value: mockFetch, writable: true });

// The static logger binds console methods as its module loads, so it is silenced by assignment rather than the flag.
Logger.IS_GLOBAL_LOGGING_ENABLED = false;
Object.assign(Logger, { debug: () => {}, error: () => {}, info: () => {}, log: () => {}, warn: () => {} });

afterEach(() => {
  resetMockInvoke();
  resetMockBulk();
  resetMockIsTauri();
  resetMockChannels();
});
