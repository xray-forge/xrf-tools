import { beforeEach, describe, expect, it } from "@jest/globals";
import { EventBus } from "@wirestate/core";
import { EConfigsCommand } from "@xrf/ipc/commands/configs";
import { SessionScopeService } from "@xrf/ipc/session";
import { mockInvoke, mockIpcCommands } from "@xrf/ipc/testing";
import { CONFIGS_JOB_VERIFY } from "@xrf/ipc/types/xrf-app-configs";
import { LtxProjectVerifyResult } from "@xrf/ipc/types/xrf-ltx";

import { IJobSettledPayload, JOB_SETTLED_EVENT } from "@/core/jobs/lib";
import { JobsService } from "@/core/jobs/services/jobs.service";
import { EMIT_NOTIFICATION_EVENT } from "@/core/notifications/lib";
import { mockInjectedService } from "@/fixtures/utils/container";
import { deferred } from "@/fixtures/utils/promise";
import { ConfigsVerifierService } from "@/tools/configs-verifier/services/configs-verifier.service";

const RESULT: LtxProjectVerifyResult = {
  outcome: "completed",
  checkedFields: 10,
  checkedSections: 2,
  duration: 1000,
  startupDuration: 50,
  findings: [],
  invalidSections: 0,
  skippedSections: 0,
  totalFiles: 1,
  totalSections: 2,
  validSections: 2,
};

describe("ConfigsVerifierService operation", () => {
  beforeEach(() => mockIpcCommands({}));

  it("preserves command arguments and emits one completion notification", async () => {
    mockIpcCommands({ [EConfigsCommand.VERIFY_DIRECTORY]: RESULT });

    const { service, container } = mockInjectedService(ConfigsVerifierService);
    const notices: Array<unknown> = [];

    container.get(EventBus).subscribe(EMIT_NOTIFICATION_EVENT, (event) => notices.push(event.payload));

    await service.verify("C:\\configs", true, true);

    expect(mockInvoke).toHaveBeenCalledWith(
      EConfigsCommand.VERIFY_DIRECTORY,
      expect.objectContaining({
        request: {
          roots: { roots: [{ path: "C:\\configs", mode: "auto" }], asset: null },
          prefix: null,
          isDltx: true,
          isStrict: true,
        },
      })
    );
    expect(service.operation.result).toEqual(RESULT);
    expect(service.operation.isRunning).toBe(false);
    expect(notices).toHaveLength(1);
  });

  it("receives an adopted result through the container event subscription", async () => {
    const { service, container } = mockInjectedService(ConfigsVerifierService);

    await container.provision();

    container.get(EventBus).emit<IJobSettledPayload>(JOB_SETTLED_EVENT, {
      id: "before-reload",
      kind: CONFIGS_JOB_VERIFY,
      result: RESULT,
      error: null,
      conclusion: "completed",
    });

    expect(service.operation.result).toEqual(RESULT);

    container.deprovision();
    container.unbindAll();
  });

  it("stops publication on deactivation while the backend still finishes and notifies", async () => {
    const response = deferred<LtxProjectVerifyResult>();

    mockIpcCommands({ [EConfigsCommand.VERIFY_DIRECTORY]: () => response.promise });

    const { service, container } = mockInjectedService(ConfigsVerifierService);
    const jobs = container.get(JobsService);
    const notices: Array<unknown> = [];

    container.get(EventBus).subscribe(EMIT_NOTIFICATION_EVENT, (event) => notices.push(event.payload));

    const running = service.verify("C:\\configs", false, false);

    // Leaving the tool takes its services and releases its scope.
    container.unbind(ConfigsVerifierService);
    container.get(SessionScopeService).release();

    expect(jobs.jobs).toHaveLength(1);

    response.resolve(RESULT);
    await running;
    await response.promise;

    expect(service.operation.result).toBeNull();
    expect(jobs.jobs).toHaveLength(0);
    expect(notices).toHaveLength(1);
  });
});
