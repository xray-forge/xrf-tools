import { Logger } from "@xrf/logging";

import { noop } from "@/lib/callbacks/noop";

/**
 * Disables application logging for tests.
 *
 */
export function mockLogger(): void {
  Logger.IS_GLOBAL_LOGGING_ENABLED = false;

  // The static logger binds console methods while the module loads, before the flag above can be flipped. Assigned
  // rather than spied on, because `jest.restoreAllMocks()` in a test's own cleanup would hand the console back.
  Object.assign(Logger, { debug: noop, error: noop, info: noop, log: noop, warn: noop });
}
