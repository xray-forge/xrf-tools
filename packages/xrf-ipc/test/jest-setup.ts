import { afterEach, jest } from "@jest/globals";
import { LOGGER_CONFIG } from "@xrf/logging";

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

LOGGER_CONFIG.setEnabled(false);

afterEach(() => {
  resetMockInvoke();
  resetMockBulk();
  resetMockIsTauri();
  resetMockChannels();
});
