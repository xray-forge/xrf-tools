import "@testing-library/jest-dom";

import { afterEach } from "@jest/globals";
import { cleanup } from "@testing-library/react";

import { resetMockAppWindow, resetMockInvoke, resetMockIsTauri } from "@/fixtures/mocks/tauri.mocks";
import { mockLogger } from "@/fixtures/setup/mock-logger";
import { mockTauri } from "@/fixtures/setup/mock-tauri";

mockLogger();
mockTauri();

afterEach(() => {
  cleanup();
  resetMockInvoke();
  resetMockIsTauri();
  resetMockAppWindow();
});
