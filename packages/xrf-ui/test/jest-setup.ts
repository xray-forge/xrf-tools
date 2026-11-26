import "@testing-library/jest-dom";

import { afterEach, jest } from "@jest/globals";
import { cleanup } from "@testing-library/react";
import { LOGGER_CONFIG } from "@xrf/logging";

// jsdom has no layout or native scrolling.
Object.defineProperty(HTMLElement.prototype, "scrollIntoView", { configurable: true, value: jest.fn() });

// What a family logs as it swallows a failure is not what its tests assert.
LOGGER_CONFIG.setEnabled(false);

afterEach(async () => {
  cleanup();

  await new Promise((resolve) => setTimeout(resolve, 0));
});
