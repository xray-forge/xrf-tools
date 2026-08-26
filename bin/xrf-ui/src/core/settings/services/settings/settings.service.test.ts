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

  it("falls back to the card grid when nothing has chosen a catalog view", () => {
    const { service } = mockInjectedService(SettingsService);

    expect(service.catalogView).toBe("grid");
  });

  it("refuses a catalog view this build does not know, rather than handing it to the launcher", () => {
    // Written by an older build, a hand edit, or a half-finished rename. `JSON.parse` would have
    // thrown on it here, taking the whole service down while it was being constructed.
    window.localStorage.setItem("xrf-catalog-view", "spreadsheet");

    const { service } = mockInjectedService(SettingsService);

    expect(service.catalogView).toBe("grid");
  });

  it("gives back the catalog view it was told to keep", () => {
    const { service } = mockInjectedService(SettingsService);

    service.setCatalogView("rows");

    expect(service.catalogView).toBe("rows");
    expect(window.localStorage.getItem("xrf-catalog-view")).toBe("rows");
    expect(mockInjectedService(SettingsService).service.catalogView).toBe("rows");
  });
});
