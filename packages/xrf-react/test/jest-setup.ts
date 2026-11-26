import "@testing-library/jest-dom";

import { afterEach } from "@jest/globals";
import { cleanup } from "@testing-library/react";
import { LOGGER_CONFIG } from "@xrf/logging";

// What a hook or boundary logs as it swallows a failure is not what its tests assert.
LOGGER_CONFIG.setEnabled(false);

afterEach(() => {
  cleanup();
});
