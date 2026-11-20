import "@testing-library/jest-dom";

import { afterEach } from "@jest/globals";
import { cleanup } from "@testing-library/react";
import { Logger } from "@xrf/logging";

function noop(): void {}

// What a hook or boundary logs as it swallows a failure is not what its tests assert. The static logger binds console
// methods while its module loads, before the flag can be flipped, so those are replaced too.
Logger.IS_GLOBAL_LOGGING_ENABLED = false;
Object.assign(Logger, { debug: noop, error: noop, info: noop, log: noop, warn: noop });

afterEach(() => {
  cleanup();
});
