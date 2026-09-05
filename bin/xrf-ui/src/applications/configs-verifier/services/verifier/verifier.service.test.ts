import { beforeEach, describe, expect, it } from "@jest/globals";
import { EventBus } from "@wirestate/core";

import { VerifierService } from "@/applications/configs-verifier/services/verifier";
import { LtxProjectVerifyResult } from "@/core/bindings/types/xrf-ltx";
import { EJobKind, IJobSettledPayload, JOB_SETTLED_EVENT } from "@/core/jobs/lib";
import { JobsService } from "@/core/jobs/services/jobs";
import { EMIT_NOTIFICATION_EVENT } from "@/core/notifications/lib";
import { mockInvoke, setMockInvokeResponses } from "@/fixtures/mocks/tauri.mocks";
import { mockInjectedService } from "@/fixtures/utils/container";
import { noop } from "@/lib/callbacks/noop";

const RESULT: LtxProjectVerifyResult = {
  outcome: "completed",
  checkedFields: 10,
  checkedSections: 2,
  duration: 1000,
  startupDuration: 50,
  errors: [],
  invalidSections: 0,
  skippedSections: 0,
  totalFiles: 1,
  totalSections: 2,
  validSections: 2,
};

describe("VerifierService operation", () => {
  beforeEach(() => setMockInvokeResponses({}));

  it("preserves command arguments and emits one completion notification", async () => {
    setMockInvokeResponses({ "plugin:configs|verify_directory": RESULT });

    const { service, container } = mockInjectedService(VerifierService);
    const notices: Array<unknown> = [];

    container.get(EventBus).subscribe(EMIT_NOTIFICATION_EVENT, (event) => notices.push(event.payload));

    await service.verify("C:\\configs", true);

    expect(mockInvoke).toHaveBeenCalledWith(
      "plugin:configs|verify_directory",
      expect.objectContaining({
        request: { roots: { roots: [{ path: "C:\\configs", mode: "auto" }], asset: null }, prefix: null, isDltx: true },
      })
    );
    expect(service.operation.result).toEqual(RESULT);
    expect(service.operation.isRunning).toBe(false);
    expect(notices).toHaveLength(1);
  });

  it("receives an adopted result through the container event subscription", async () => {
    const { service, container } = mockInjectedService(VerifierService);

    await container.provision();

    container.get(EventBus).emit<IJobSettledPayload>(JOB_SETTLED_EVENT, {
      id: "before-reload",
      kind: EJobKind.CONFIGS_VERIFY,
      result: RESULT,
      error: null,
      conclusion: "completed",
    });

    expect(service.operation.result).toEqual(RESULT);

    container.deprovision();
    container.unbindAll();
  });

  it("stops publication on deactivation while the backend still finishes and notifies", async () => {
    let finish: (result: LtxProjectVerifyResult) => void = noop;
    const response = new Promise<LtxProjectVerifyResult>((resolve) => {
      finish = resolve;
    });

    setMockInvokeResponses({ "plugin:configs|verify_directory": () => response });

    const { service, container } = mockInjectedService(VerifierService);
    const jobs = container.get(JobsService);
    const notices: Array<unknown> = [];

    container.get(EventBus).subscribe(EMIT_NOTIFICATION_EVENT, (event) => notices.push(event.payload));

    const running = service.verify("C:\\configs", false);

    container.unbind(VerifierService);

    expect(jobs.jobs).toHaveLength(1);

    finish(RESULT);
    await running;
    await response;

    expect(service.operation.result).toBeNull();
    expect(jobs.jobs).toHaveLength(0);
    expect(notices).toHaveLength(1);
  });
});
