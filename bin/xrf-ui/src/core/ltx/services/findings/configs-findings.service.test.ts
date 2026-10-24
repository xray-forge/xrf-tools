import { describe, expect, it } from "@jest/globals";
import { setMockInvokeResponses } from "@xrf/ipc/testing";
import { LtxAnchoredFinding } from "@xrf/ipc/types/xrf-ltx-inspect";

import { ConfigsFindingsService } from "@/core/ltx/services/findings";
import { ConfigsProjectService } from "@/core/ltx/services/project";
import { mockRestoredConfigsProject } from "@/fixtures/utils/configs-project";
import { mockInjectedService } from "@/fixtures/utils/container";
import { noop } from "@/lib/callbacks/noop";

describe("ConfigsFindingsService", () => {
  it("keeps cleared findings idle when verification finishes", async () => {
    let resolve: (findings: Array<LtxAnchoredFinding>) => void = noop;
    const response = new Promise<Array<LtxAnchoredFinding>>((settle) => {
      resolve = settle;
    });

    setMockInvokeResponses({ "plugin:configs|list_findings": () => response });

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
