import { jest } from "@jest/globals";

import { mockAppWindow, MockChannel, mockInvoke, mockIsTauri } from "@/fixtures/mocks/tauri.mocks";

/**
 * Registers the Tauri API mocks used by jsdom tests.
 */
export function mockTauri(): void {
  jest.mock("@tauri-apps/api/core", () => ({
    invoke: mockInvoke,
    convertFileSrc: (path: string) => `asset://${path}`,
    isTauri: mockIsTauri,
    Channel: MockChannel,
  }));

  jest.mock("@tauri-apps/api/window", () => ({
    getCurrentWindow: () => mockAppWindow,
  }));

  jest.mock("@tauri-apps/api", () => ({
    path: {
      dirname: async (value: string) => value.slice(0, value.lastIndexOf("\\")),
      join: async (...parts: Array<string>) => parts.join("\\"),
      resolve: async (...parts: Array<string>) => parts.join("\\"),
    },
  }));

  // The submodule as well as the root, since production code imports `@tauri-apps/api/path` directly and its real
  // implementation would go to the backend for every join - answering null through the invoke harness.
  jest.mock("@tauri-apps/api/path", () => ({
    dirname: async (value: string) => value.slice(0, value.lastIndexOf("\\")),
    join: async (...parts: Array<string>) => parts.join("\\"),
    resolve: async (...parts: Array<string>) => parts.join("\\"),
  }));

  jest.mock("@tauri-apps/plugin-dialog", () => ({
    open: jest.fn(async () => null),
    save: jest.fn(async () => null),
  }));

  jest.mock("@tauri-apps/plugin-fs", () => ({
    exists: jest.fn(async () => true),
  }));

  jest.mock("@tauri-apps/plugin-shell", () => ({
    open: jest.fn(async () => undefined),
  }));
}
