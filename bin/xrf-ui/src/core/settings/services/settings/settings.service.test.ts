import { beforeEach, describe, expect, it } from "@jest/globals";

import { SettingsService } from "@/core/settings/services/settings/settings.service";
import { mockInjectedService } from "@/fixtures/utils/container";

describe("SettingsService", () => {
  beforeEach(() => {
    window.localStorage.clear();
  });

  it("takes its first value from the build, which is on under test", () => {
    const { service } = mockInjectedService(SettingsService);

    // Jest runs with NODE_ENV=test, so `isDevelopmentBuild()` is true here for the same reason it is
    // true in a dev bundle: only a production build turns it off.
    expect(service.isDevModeEnabled).toBe(true);
  });

  it("lets a stored choice override the build default", () => {
    window.localStorage.setItem("xrf-dev-mode", "false");

    const { service } = mockInjectedService(SettingsService);

    expect(service.isDevModeEnabled).toBe(false);
  });

  it("persists what was chosen, which is what makes tracing a release build possible", () => {
    const { service } = mockInjectedService(SettingsService);

    service.setDevModeEnabled(false);

    expect(service.isDevModeEnabled).toBe(false);
    expect(window.localStorage.getItem("xrf-dev-mode")).toBe("false");
    expect(service.isDevModeEnabled).toBe(false);
  });
});
