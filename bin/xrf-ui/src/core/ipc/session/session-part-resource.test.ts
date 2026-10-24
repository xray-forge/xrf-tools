import { describe, expect, it, jest } from "@jest/globals";
import { EventBus } from "@wirestate/core";

import { ESessionRootChange, SessionChange, SessionSnapshot } from "@/core/ipc/types/xrf-app-core";
import { mockContainer } from "@/fixtures/utils/container";

import { SESSION_CHANGE_EVENT, SessionPartResource } from "./index";

const PART: string = "levelMap";

function snapshot(value: string, revision: number, sessionId: string = "level"): SessionSnapshot<string> {
  return { sessionId, revision, value };
}

function kept(revision: number, staleParts: Array<string>, sessionId: string = "level"): SessionChange {
  return { revision, root: { kind: ESessionRootChange.KEPT }, sessionId, staleParts, isViewRenewed: false };
}

async function flush(): Promise<void> {
  for (let index: number = 0; index < 5; index += 1) {
    await Promise.resolve();
  }
}

describe("SessionPartResource", () => {
  it("reads its part as it attaches, and once again for a change naming it", async () => {
    const bus: EventBus = mockContainer().get(EventBus);
    const query = jest
      .fn<(sessionId: string) => Promise<SessionSnapshot<string>>>()
      .mockResolvedValueOnce(snapshot("first", 0))
      .mockResolvedValueOnce(snapshot("second", 1));
    const resource: SessionPartResource<string> = new SessionPartResource(bus, [PART], query);

    await resource.attach("level");

    expect(resource.value).toBe("first");

    bus.emit<SessionChange>(SESSION_CHANGE_EVENT, kept(1, [PART]));
    await flush();

    expect(query).toHaveBeenCalledTimes(2);
    expect(resource.value).toBe("second");
    expect(resource.snapshot?.revision).toBe(1);
  });

  it("reads nothing again for another part, another opening, or a change it holds already", async () => {
    const bus: EventBus = mockContainer().get(EventBus);
    const query = jest
      .fn<(sessionId: string) => Promise<SessionSnapshot<string>>>()
      .mockResolvedValue(snapshot("held", 2));
    const resource: SessionPartResource<string> = new SessionPartResource(bus, [PART], query);

    await resource.attach("level");

    bus.emit<SessionChange>(SESSION_CHANGE_EVENT, kept(3, ["spawn"]));
    bus.emit<SessionChange>(SESSION_CHANGE_EVENT, kept(3, [PART], "another"));
    bus.emit<SessionChange>(SESSION_CHANGE_EVENT, kept(2, [PART]));
    await flush();

    expect(query).toHaveBeenCalledTimes(1);
  });

  it("reads again for an opening derived anew, which lets every part go", async () => {
    const bus: EventBus = mockContainer().get(EventBus);
    const query = jest
      .fn<(sessionId: string) => Promise<SessionSnapshot<string>>>()
      .mockResolvedValueOnce(snapshot("first", 0))
      .mockResolvedValueOnce(snapshot("derived", 1));
    const resource: SessionPartResource<string> = new SessionPartResource(bus, [PART], query);

    await resource.attach("level");

    bus.emit<SessionChange>(SESSION_CHANGE_EVENT, {
      revision: 1,
      root: { kind: ESessionRootChange.DERIVED },
      sessionId: "level",
      staleParts: [],
      isViewRenewed: true,
    });
    await flush();

    expect(resource.value).toBe("derived");
  });

  it("drops an answer older than the one held, and one for an opening let go", async () => {
    const bus: EventBus = mockContainer().get(EventBus);
    const answers: Array<(answer: SessionSnapshot<string>) => void> = [];
    const query = jest.fn<(sessionId: string) => Promise<SessionSnapshot<string>>>(
      () => new Promise((resolve) => answers.push(resolve))
    );
    const resource: SessionPartResource<string> = new SessionPartResource(bus, [PART], query);

    const attached: Promise<void> = resource.attach("level");

    answers[0](snapshot("newer", 3));
    await attached;

    bus.emit<SessionChange>(SESSION_CHANGE_EVENT, kept(4, [PART]));
    answers[1](snapshot("older", 2));
    await flush();

    expect(resource.value).toBe("newer");

    bus.emit<SessionChange>(SESSION_CHANGE_EVENT, kept(5, [PART]));
    resource.detach();
    answers[2](snapshot("late", 5));
    await flush();

    expect(resource.value).toBeNull();
  });

  it("keeps what it holds while the opening's own files move, and where a read fails", async () => {
    const bus: EventBus = mockContainer().get(EventBus);
    const query = jest
      .fn<(sessionId: string) => Promise<SessionSnapshot<string>>>()
      .mockResolvedValueOnce(snapshot("shown", 0))
      .mockRejectedValueOnce(new Error("cannot read"));
    const resource: SessionPartResource<string> = new SessionPartResource(bus, [PART], query);

    await resource.attach("level");

    bus.emit<SessionChange>(SESSION_CHANGE_EVENT, {
      revision: 0,
      root: { kind: ESessionRootChange.DERIVING },
      sessionId: "level",
      staleParts: [],
      isViewRenewed: false,
    });

    expect(resource.value).toBe("shown");
    expect(query).toHaveBeenCalledTimes(1);

    bus.emit<SessionChange>(SESSION_CHANGE_EVENT, kept(1, [PART]));
    await flush();

    expect(resource.value).toBe("shown");
    expect(resource.error?.message).toBe("cannot read");
  });

  // An open's first read overtaken by its catch-up: whoever waits on the attach finds the part read, not dropped.
  it("settles an attach once the read overtaking its own lands", async () => {
    const bus: EventBus = mockContainer().get(EventBus);
    const answers: Array<(answer: SessionSnapshot<string>) => void> = [];
    const query = jest.fn<(sessionId: string) => Promise<SessionSnapshot<string>>>(
      () => new Promise((resolve) => answers.push(resolve))
    );
    const resource: SessionPartResource<string> = new SessionPartResource(bus, [PART], query);
    let isSettled: boolean = false;

    const attached: Promise<void> = resource.attach("level").then(() => {
      isSettled = true;
    });

    bus.emit<SessionChange>(SESSION_CHANGE_EVENT, {
      revision: 1,
      root: { kind: ESessionRootChange.DERIVED },
      sessionId: "level",
      staleParts: [],
      isViewRenewed: true,
    });
    answers[0](snapshot("overtaken", 0));
    await flush();

    expect(isSettled).toBe(false);

    answers[1](snapshot("derived", 1));
    await attached;

    expect(resource.value).toBe("derived");
  });

  it("hears no change once let go", async () => {
    const bus: EventBus = mockContainer().get(EventBus);
    const query = jest
      .fn<(sessionId: string) => Promise<SessionSnapshot<string>>>()
      .mockResolvedValue(snapshot("held", 0));
    const resource: SessionPartResource<string> = new SessionPartResource(bus, [PART], query);

    await resource.attach("level");
    resource.detach();

    bus.emit<SessionChange>(SESSION_CHANGE_EVENT, kept(1, [PART]));
    await flush();

    expect(query).toHaveBeenCalledTimes(1);
    expect(resource.value).toBeNull();
  });
});
