import "@testing-library/jest-dom";

import { afterEach, jest } from "@jest/globals";
import { cleanup } from "@testing-library/react";

import { resetMockBulk } from "@/fixtures/mocks/bulk.mocks";
import { resetMockAppWindow, resetMockInvoke, resetMockIsTauri } from "@/fixtures/mocks/tauri.mocks";
import { mockLogger } from "@/fixtures/setup/mock-logger";
import { mockTauri } from "@/fixtures/setup/mock-tauri";
import { mockTransport } from "@/fixtures/setup/mock-transport";

mockLogger();
mockTauri();
mockTransport();

// jsdom has no layout or native scrolling.
Object.defineProperty(HTMLElement.prototype, "scrollIntoView", { configurable: true, value: jest.fn() });

afterEach(async () => {
  cleanup();

  await new Promise((resolve) => setTimeout(resolve, 0));

  resetMockInvoke();
  resetMockBulk();
  resetMockIsTauri();
  resetMockAppWindow();
});
