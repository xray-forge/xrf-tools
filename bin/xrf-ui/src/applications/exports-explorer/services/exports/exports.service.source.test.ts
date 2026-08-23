import { beforeEach, describe, expect, it } from "@jest/globals";

import { ExportsService } from "@/applications/exports-explorer/services/exports/exports.service";
import { ExportSourceContent } from "@/core/bindings/types/xrf-export";
import { mockInvoke, setMockInvokeResponses } from "@/fixtures/mocks/tauri.mocks";
import { mockInjectedService } from "@/fixtures/utils/container";

const SOURCE: ExportSourceContent = {
  name: "xr_effects.play",
  path: "effects/sound.ts",
  line: 18,
  endLine: 21,
  content: 'extern("xr_effects.play", (): void => {});',
};

describe("ExportsService export source", () => {
  beforeEach(() => {
    setMockInvokeResponses({ ["plugin:exports|get_source"]: SOURCE });
  });

  it("reads the source of one declaration by name", async () => {
    const service: ExportsService = mockInjectedService(ExportsService).service;

    await expect(service.readExportSource("xr_effects.play")).resolves.toEqual(SOURCE);
    expect(mockInvoke).toHaveBeenCalledWith("plugin:exports|get_source", { name: "xr_effects.play" });
  });

  it("propagates a failed read to its caller", async () => {
    // Reporting is the view's job here, so the service must not swallow this into a null result.
    const service: ExportsService = mockInjectedService(ExportsService).service;

    setMockInvokeResponses({
      ["plugin:exports|get_source"]: () => {
        throw new Error("declaration file is gone");
      },
    });

    await expect(service.readExportSource("xr_effects.play")).rejects.toThrow("declaration file is gone");
  });

  it("holds no source state of its own", () => {
    // The body belongs to whatever is on screen; keeping it here would make the service arbitrate
    // between in-flight reads that the viewing effect already knows how to abandon.
    expect(mockInjectedService(ExportsService).service).not.toHaveProperty("source");
  });
});
