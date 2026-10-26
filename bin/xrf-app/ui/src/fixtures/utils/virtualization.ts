import { dirname } from "node:path";

/**
 * Lifts `@mui/x-virtualizer`'s own opt-out, which turns windowing off under a test runner.
 *
 * It reads `platform.env.jsdom` once, when a virtualizer's store is created. The flag's module is resolved from the
 * virtualizer's own location, so the instance flipped is the one it reads, whatever the package manager hoists.
 *
 * @param isEnabled - Whether the virtualizer should behave as it does in a browser.
 */
export function setVirtualizationEnabled(isEnabled: boolean): void {
  const path: string = require.resolve("@base-ui/utils/platform", {
    paths: [dirname(require.resolve("@mui/x-virtualizer"))],
  });
  const { platform }: { platform: { env: { jsdom: boolean } } } = require(path);

  platform.env.jsdom = !isEnabled;
}
