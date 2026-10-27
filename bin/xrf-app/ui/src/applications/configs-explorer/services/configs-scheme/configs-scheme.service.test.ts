import { describe, expect, it } from "@jest/globals";
import { EConfigsCommand } from "@xrf/ipc/commands/configs";
import { mockIpcCommands } from "@xrf/ipc/testing";
import { LtxSectionSchemeReport } from "@xrf/ipc/types/xrf-ltx-inspect";
import { Nullable } from "@xrf/types";

import { mockRestoredConfigsProject } from "@/applications/configs-explorer/fixtures/configs-project";
import { ConfigsProjectService } from "@/applications/configs-explorer/services/configs-project";
import { ConfigsSchemeService } from "@/applications/configs-explorer/services/configs-scheme";
import { mockSessionRead } from "@/fixtures/mocks/session.mocks";
import { mockInjectedService } from "@/fixtures/utils/container";
import { deferred } from "@/fixtures/utils/promise";

describe("ConfigsSchemeService", () => {
  it("keeps a cleared report idle when its read finishes", async () => {
    const response = deferred<Nullable<LtxSectionSchemeReport>>();

    mockIpcCommands({ [EConfigsCommand.READ_SECTION_SCHEME]: mockSessionRead(() => response.promise) });

    const { service, container } = mockInjectedService(ConfigsSchemeService, [ConfigsProjectService]);

    await mockRestoredConfigsProject(container.get(ConfigsProjectService));

    const reading = service.read("system.ltx", "wpn_base");

    service.clear();
    response.resolve(null);
    await reading;

    expect(service.entry).toBeNull();
    expect(service.section).toBeNull();
    expect(service.report.isIdle).toBe(true);
    expect(service.report.value).toBeNull();
  });
});
