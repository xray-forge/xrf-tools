import { describe, expect, it } from "@jest/globals";
import { EConfigsCommand } from "@xrf/ipc/commands/configs";
import { mockIpcCommands } from "@xrf/ipc/testing";
import { LtxSectionSchemeReport } from "@xrf/ipc/types/xrf-ltx-inspect";
import { Nullable } from "@xrf/types";

import { ConfigsProjectService } from "@/core/ltx/services/project";
import { ConfigsSchemeService } from "@/core/ltx/services/scheme";
import { mockSessionRead } from "@/fixtures/mocks/session.mocks";
import { mockRestoredConfigsProject } from "@/fixtures/utils/configs-project";
import { mockInjectedService } from "@/fixtures/utils/container";
import { noop } from "@/lib/callbacks/noop";

describe("ConfigsSchemeService", () => {
  it("keeps a cleared report idle when its read finishes", async () => {
    let resolve: (report: Nullable<LtxSectionSchemeReport>) => void = noop;
    const response = new Promise<Nullable<LtxSectionSchemeReport>>((settle) => {
      resolve = settle;
    });

    mockIpcCommands({ [EConfigsCommand.READ_SECTION_SCHEME]: mockSessionRead(() => response) });

    const { service, container } = mockInjectedService(ConfigsSchemeService, [ConfigsProjectService]);

    await mockRestoredConfigsProject(container.get(ConfigsProjectService));

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
