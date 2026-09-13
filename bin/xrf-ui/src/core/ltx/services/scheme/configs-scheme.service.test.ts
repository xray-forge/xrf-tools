import { describe, expect, it } from "@jest/globals";

import { ConfigsProjectDescriptor } from "@/core/ipc/types/xrf-app";
import { LtxSectionSchemeReport } from "@/core/ipc/types/xrf-ltx-inspect";
import { ConfigsProjectService } from "@/core/ltx/services/project";
import { ConfigsSchemeService } from "@/core/ltx/services/scheme";
import { setMockInvokeResponses } from "@/fixtures/mocks/tauri.mocks";
import { mockInjectedService } from "@/fixtures/utils/container";
import { noop } from "@/lib/callbacks/noop";
import { Nullable } from "@/lib/types/general";

describe("ConfigsSchemeService", () => {
  it("keeps a cleared report idle when its read finishes", async () => {
    let resolve: (report: Nullable<LtxSectionSchemeReport>) => void = noop;
    const response = new Promise<Nullable<LtxSectionSchemeReport>>((settle) => {
      resolve = settle;
    });

    setMockInvokeResponses({ "plugin:configs|read_section_scheme": () => response });

    const { service, container } = mockInjectedService(ConfigsSchemeService, [ConfigsProjectService]);
    const project = container.get(ConfigsProjectService);

    project.project = project.project.asReady({ sessionId: "session-1" } as ConfigsProjectDescriptor);

    const reading = service.read("system.ltx", "wpn_base");

    service.clear();
    resolve(null);
    await reading;

    expect(service.entry).toBeNull();
    expect(service.section).toBeNull();
    expect(service.report.isIdle).toBe(true);
    expect(service.report.value).toBeNull();
  });
});
