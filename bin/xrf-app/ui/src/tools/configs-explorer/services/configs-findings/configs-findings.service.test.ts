import { describe, expect, it } from "@jest/globals";
import { EConfigsCommand } from "@xrf/ipc/commands/configs";
import { mockIpcCommands } from "@xrf/ipc/testing";
import { LtxAnchoredFinding } from "@xrf/ipc/types/xrf-ltx-inspect";

import { mockSessionRead } from "@/fixtures/mocks/session.mocks";
import { mockInjectedService } from "@/fixtures/utils/container";
import { deferred } from "@/fixtures/utils/promise";
import { mockRestoredConfigsProject } from "@/tools/configs-explorer/fixtures/configs-project";
import { ConfigsFindingsService } from "@/tools/configs-explorer/services/configs-findings";
import { ConfigsProjectService } from "@/tools/configs-explorer/services/configs-project";

describe("ConfigsFindingsService", () => {
  it("keeps cleared findings idle when verification finishes", async () => {
    const response = deferred<Array<LtxAnchoredFinding>>();

    mockIpcCommands({ [EConfigsCommand.LIST_FINDINGS]: mockSessionRead(() => response.promise) });

    const { service, container } = mockInjectedService(ConfigsFindingsService, [ConfigsProjectService]);

    await mockRestoredConfigsProject(container.get(ConfigsProjectService));

    const verifying = service.open("system.ltx");

    service.clear();
    response.resolve([]);
    await verifying;

    expect(service.entry).toBeNull();
    expect(service.findings.isIdle).toBe(true);
    expect(service.findings.value).toEqual([]);
  });
});
