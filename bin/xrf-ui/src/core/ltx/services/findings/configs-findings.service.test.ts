import { describe, expect, it } from "@jest/globals";

import { ConfigsProjectDescriptor } from "@/core/ipc/types/xrf-app";
import { LtxAnchoredFinding } from "@/core/ipc/types/xrf-ltx-inspect";
import { ConfigsFindingsService } from "@/core/ltx/services/findings";
import { ConfigsProjectService } from "@/core/ltx/services/project";
import { setMockInvokeResponses } from "@/fixtures/mocks/tauri.mocks";
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
    const project = container.get(ConfigsProjectService);

    project.project = project.project.asReady({ sessionId: "session-1" } as ConfigsProjectDescriptor);

    const verifying = service.open("system.ltx");

    service.clear();
    resolve([]);
    await verifying;

    expect(service.entry).toBeNull();
    expect(service.findings.isIdle).toBe(true);
    expect(service.findings.value).toEqual([]);
  });
});
