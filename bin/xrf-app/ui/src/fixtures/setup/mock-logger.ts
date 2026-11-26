import { LOGGER_CONFIG } from "@xrf/logging";

/**
 * Silences every logger for tests, the static methods and loggers made earlier included; a test that asserts what is
 * logged turns logging back on for itself.
 */
export function mockLogger(): void {
  LOGGER_CONFIG.setEnabled(false);
}
