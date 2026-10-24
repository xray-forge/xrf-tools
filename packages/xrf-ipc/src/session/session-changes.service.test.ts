import { beforeEach, describe, expect, it } from "@jest/globals";
import { EventBus, WireEvent } from "@wirestate/core";

import { mockInjectedService } from "../../test/container";

import { SessionChangesService } from "#/session/session-changes.service";
import { SESSION_CHANGE_EVENT } from "#/session/session.types";
import {
  MockChannel,
  mockInvoke,
  mockIsTauri,
  resetMockInvoke,
  resetMockIsTauri,
  setMockInvokeResponses,
} from "#/testing/index";
import { AssetChangeEvent, SessionChange } from "#/types/xrf-app-core";

const SUBSCRIBE: string = "plugin:assets|subscribe_changes";

function countSubscriptions(): number {
  return mockInvoke.mock.calls.filter(([name]) => name === SUBSCRIBE).length;
}

function getChanges(): MockChannel<AssetChangeEvent> {
  const call = mockInvoke.mock.calls.filter(([name]) => name === SUBSCRIBE).at(-1);

  return (call?.[1] as { changes: MockChannel<AssetChangeEvent> }).changes;
}

describe("SessionChangesService", () => {
  beforeEach(() => {
    resetMockInvoke();
    resetMockIsTauri();
  });

  it("subscribes the window once, however often the shell asks", async () => {
    const { service } = mockInjectedService(SessionChangesService);

    await Promise.all([service.listen(), service.listen()]);
    await service.listen();

    expect(countSubscriptions()).toBe(1);
  });

  it("tells each session a change reached as a notice of its own", async () => {
    const { container, service } = mockInjectedService(SessionChangesService);
    const notices: Array<SessionChange> = [];
    const level: SessionChange = {
      sessionId: "level",
      revision: 3,
      root: { kind: "kept" },
      staleParts: ["spawn"],
      isViewRenewed: false,
    };
    const project: SessionChange = {
      sessionId: "project",
      revision: 3,
      root: { kind: "derived" },
      staleParts: [],
      isViewRenewed: true,
    };

    container
      .get(EventBus)
      .subscribe(SESSION_CHANGE_EVENT, (event: WireEvent<SessionChange>) =>
        notices.push(event.payload as SessionChange)
      );

    await service.listen();

    getChanges().onmessage({ revision: 3, sessions: [level, project] });

    expect(notices).toEqual([level, project]);
  });

  it("subscribes again after the backend refused, and nowhere outside the desktop shell", async () => {
    const { service } = mockInjectedService(SessionChangesService);

    setMockInvokeResponses({ [SUBSCRIBE]: () => Promise.reject(new Error("no window")) });

    await service.listen();

    setMockInvokeResponses({});

    await service.listen();

    expect(countSubscriptions()).toBe(2);

    const { service: browser } = mockInjectedService(SessionChangesService);

    mockIsTauri.mockReturnValue(false);

    await browser.listen();

    expect(countSubscriptions()).toBe(2);
  });
});
