import { describe, expect, it } from "@jest/globals";
import { EConfigsCommand } from "@xrf/ipc/commands/configs";
import { mockIpcCommands } from "@xrf/ipc/testing";
import { LtxAnchoredFinding } from "@xrf/ipc/types/xrf-ltx-inspect";

import { mockRestoredConfigsProject } from "@/core/ltx/fixtures/configs-project";
import { ConfigsFindingsService } from "@/core/ltx/services/findings";
import { ConfigsProjectService } from "@/core/ltx/services/project";
import { mockSessionRead } from "@/fixtures/mocks/session.mocks";
import { mockInjectedService } from "@/fixtures/utils/container";
import { noop } from "@/lib/callbacks/noop";

describe("ConfigsFindingsService", () => {
  it("keeps cleared findings idle when verification finishes", async () => {
    let resolve: (findings: Array<LtxAnchoredFinding>) => void = noop;
    const response = new Promise<Array<LtxAnchoredFinding>>((settle) => {
      resolve = settle;
    });

    mockIpcCommands({ [EConfigsCommand.LIST_FINDINGS]: mockSessionRead(() => response) });

    const { service, container } = mockInjectedService(ConfigsFindingsService, [ConfigsProjectService]);

    await mockRestoredConfigsProject(container.get(ConfigsProjectService));

    const verifying = service.open("system.ltx");

    service.clear();
    resolve([]);
    await verifying;

    expect(service.entry).toBeNull();
    expect(service.findings.isIdle).toBe(true);
    expect(service.findings.value).toEqual([]);
  });
});
