import { beforeEach, describe, expect, it, jest } from "@jest/globals";
import { Container, EventBus, Injectable, WireEvent } from "@wirestate/core";
import { flowResult } from "@wirestate/mobx";
import { SessionScopeService } from "@xrf/ipc/session";
import { mockIpcCommands } from "@xrf/ipc/testing";
import { SessionId, SessionRestore, SessionSnapshot } from "@xrf/ipc/types/xrf-app-core";
import { Logger } from "@xrf/logging";

import { EMIT_NOTIFICATION_EVENT, ENotificationSeverity, INotificationPayload } from "@/core/notifications/lib";
import { mockSessionSnapshot } from "@/fixtures/mocks/session.mocks";
import { mockApplicationScope, mockInjectedService } from "@/fixtures/utils/container";
import { noop } from "@/lib/callbacks/noop";

import { SessionRootResource } from "./session-root-resource";

const close = jest.fn<(sessionIds: Array<SessionId>) => Promise<unknown>>();
const restore = jest.fn<(tool: string) => Promise<SessionRestore<string>>>();
const open = jest.fn<(sessionId: SessionId, path: string) => Promise<SessionSnapshot<string>>>();

@Injectable()
class HolderService {
  public readonly log: Logger = new Logger("holder");

  public readonly root: SessionRootResource<string> = new SessionRootResource(this.log, {
    subject: "test project",
    close,
    restore,
  });
}

@Injectable()
class UnrestoredHolderService {
  public readonly log: Logger = new Logger("holder");

  public readonly root: SessionRootResource<string> = new SessionRootResource(this.log, {
    subject: "test project",
    close,
  });
}

function deferred<T>() {
  let resolve: (value: T) => void = noop;

  const promise = new Promise<T>((settle) => {
    resolve = settle;
  });

  return { promise, resolve };
}

function setup() {
  const { container, service } = mockInjectedService(HolderService, mockApplicationScope("exports-explorer"));
  const raised: Array<INotificationPayload> = [];

  container
    .get(EventBus)
    .subscribe(EMIT_NOTIFICATION_EVENT, (event: WireEvent<INotificationPayload>) =>
      raised.push(event.payload as INotificationPayload)
    );

  return { container, raised, root: service.root };
}

async function flush(): Promise<void> {
  for (let index: number = 0; index < 5; index += 1) {
    await Promise.resolve();
  }
}

function opened(value: string) {
  return async (sessionId: SessionId): Promise<SessionSnapshot<string>> => mockSessionSnapshot(value, sessionId);
}

describe("SessionRootResource", () => {
  beforeEach(() => {
    mockIpcCommands({});
    close.mockReset().mockResolvedValue(undefined);
    restore.mockReset().mockResolvedValue(null);
    open.mockReset().mockImplementation(opened("opened"));
  });

  it("restores what its tool holds, and owns it, so a close releases it", async () => {
    const { root } = setup();

    restore.mockResolvedValue(mockSessionSnapshot("restored", "held"));

    await flowResult(root.restore());

    expect(restore).toHaveBeenCalledWith("exports-explorer");
    expect(root.isRestored).toBe(true);
    expect(root.value).toBe("restored");
    expect(root.sessionId).toBe("held");

    await expect(flowResult(root.close())).resolves.toBe(true);

    expect(close).toHaveBeenCalledWith(["held"]);
    expect(root.value).toBeNull();
    expect(root.state.isIdle).toBe(true);
  });

  it("shows a failed restore and warns of it, and is restored all the same", async () => {
    const { raised, root } = setup();

    restore.mockRejectedValue(new Error("backend unavailable"));

    await flowResult(root.restore());

    expect(root.isRestored).toBe(true);
    expect(root.error).toEqual(new Error("backend unavailable"));
    expect(raised).toEqual([
      {
        details: "backend unavailable",
        severity: ENotificationSeverity.WARNING,
        source: "exports-explorer",
        title: "Could not restore the open test project",
      },
    ]);
  });

  it("is restored at once where nothing is ever restored", async () => {
    const { service } = mockInjectedService(UnrestoredHolderService);

    await flowResult(service.root.restore());

    expect(service.root.isRestored).toBe(true);
    expect(service.root.state.isIdle).toBe(true);
  });

  it("never lets a restore publish over an open under way", async () => {
    const { root } = setup();
    const answer = deferred<SessionSnapshot<string>>();

    open.mockImplementation(() => answer.promise);
    restore.mockResolvedValue(mockSessionSnapshot("restored", "held"));

    const opening = flowResult(root.open("C:\\project", open, "C:\\project"));
    const restoring = flowResult(root.restore());

    answer.resolve(mockSessionSnapshot("opened", "fresh"));
    await Promise.all([opening, restoring]);

    expect(restore).not.toHaveBeenCalled();
    expect(root.value).toBe("opened");
    expect(root.isRestored).toBe(true);
  });

  it("opens in place of what it holds, a newer open replacing an older one", async () => {
    const { root } = setup();
    const first = deferred<SessionSnapshot<string>>();

    open.mockImplementationOnce((sessionId: SessionId) =>
      first.promise.then((snapshot) => ({ ...snapshot, sessionId }))
    );

    const older = flowResult(root.open("C:\\first", open, "C:\\first"));
    const newer = flowResult(root.open("C:\\second", open, "C:\\second"));

    await newer;
    first.resolve(mockSessionSnapshot("first"));
    await older;
    // The cancelled flow settles at once; the session releases what it then answers in its own time.
    await flush();

    expect(root.value).toBe("opened");
    expect(open).toHaveBeenCalledTimes(2);
    // The older opening lands after the newer published, so it is released rather than shown.
    expect(close).toHaveBeenCalledWith([open.mock.calls[0][0]]);
  });

  it("keeps what it holds where an open fails, and raises the failure naming what was opened", async () => {
    const { raised, root } = setup();

    await flowResult(root.open("C:\\first", open, "C:\\first"));

    open.mockRejectedValue(new Error("no scripts directory"));

    await flowResult(root.open("C:\\second", open, "C:\\second"));

    expect(root.value).toBe("opened");
    expect(root.isLoading).toBe(false);
    expect(root.error).toEqual(new Error("no scripts directory"));
    expect(raised).toEqual([
      {
        details: "C:\\second\nno scripts directory",
        severity: ENotificationSeverity.ERROR,
        source: "exports-explorer",
        title: "Could not open the test project",
      },
    ]);
  });

  it("refreshes only what it holds, keeping it where the refresh fails", async () => {
    const { raised, root } = setup();

    await flowResult(root.refresh("C:\\project", open, "C:\\project"));

    expect(open).not.toHaveBeenCalled();

    await flowResult(root.open("C:\\project", open, "C:\\project"));

    open.mockRejectedValue(new Error("scripts moved"));

    await flowResult(root.refresh("C:\\project", open, "C:\\project"));

    expect(root.value).toBe("opened");
    expect(root.error).toEqual(new Error("scripts moved"));
    expect(raised.map((notice) => notice.title)).toEqual(["Could not refresh the test project"]);
  });

  it("keeps what it holds where a close fails, and raises the failure", async () => {
    const { raised, root } = setup();

    await flowResult(root.open("C:\\project", open, "C:\\project"));

    close.mockRejectedValue(new Error("project is busy"));

    await expect(flowResult(root.close())).resolves.toBe(false);

    expect(root.value).toBe("opened");
    expect(root.isLoading).toBe(false);
    expect(raised.map((notice) => notice.title)).toEqual(["Could not close the test project"]);
  });

  it("settles what it holds where a close that overtook an open fails", async () => {
    const { root } = setup();
    const answer = deferred<SessionSnapshot<string>>();

    await flowResult(root.open("C:\\first", open, "C:\\first"));

    open.mockImplementation(() => answer.promise);
    close.mockRejectedValue(new Error("project is busy"));

    const opening = flowResult(root.open("C:\\second", open, "C:\\second"));

    await expect(flowResult(root.close())).resolves.toBe(false);
    answer.resolve(mockSessionSnapshot("second", "late"));
    await opening;

    expect(root.value).toBe("opened");
    expect(root.isLoading).toBe(false);
  });

  it("publishes nothing once its tool is left, and forgets what it held", async () => {
    const { container, raised, root } = setup();
    const answer = deferred<SessionSnapshot<string>>();

    await flowResult(root.open("C:\\first", open, "C:\\first"));

    open.mockImplementation(() => answer.promise);

    const opening = flowResult(root.open("C:\\second", open, "C:\\second"));

    (container as Container).get(SessionScopeService).release();
    answer.resolve(mockSessionSnapshot("second", "late"));
    await opening;

    expect(root.value).toBeNull();
    expect(root.isLoading).toBe(false);
    expect(raised).toEqual([]);
  });
});
