import "@testing-library/jest-dom";

import { afterEach, jest } from "@jest/globals";
import { cleanup } from "@testing-library/react";
import { Logger } from "@xrf/logging";

function noop(): void {}

// jsdom has no layout or native scrolling.
Object.defineProperty(HTMLElement.prototype, "scrollIntoView", { configurable: true, value: jest.fn() });

// What a family logs as it swallows a failure is not what its tests assert. The static logger binds console methods
// while its module loads, before the flag can be flipped, so those are replaced too.
Logger.IS_GLOBAL_LOGGING_ENABLED = false;
Object.assign(Logger, { debug: noop, error: noop, info: noop, log: noop, warn: noop });

afterEach(async () => {
  cleanup();

  await new Promise((resolve) => setTimeout(resolve, 0));
});
