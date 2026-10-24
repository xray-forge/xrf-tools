import { beforeEach, describe, expect, it, jest } from "@jest/globals";
import { Logger } from "@xrf/logging";

import { ESessionsCommand } from "#/commands/sessions";
import { ISessionIdentity, Session, SessionScopeService } from "#/session/index";
import { mockInvoke, mockIsTauri, resetMockIsTauri } from "#/testing/index";

describe("Session", () => {
  beforeEach(resetMockIsTauri);

  it("releases an opening before its response and releases a late publication again", async () => {
    const release = jest.fn<(ids: Array<string>) => Promise<void>>().mockResolvedValue(undefined);
    const session = new Session(release, new SessionScopeService("test"));

    let finish!: (value: ISessionIdentity) => void;
    let id: string = "";

    const command = jest.fn<(sessionId: string, path: string) => Promise<ISessionIdentity>>((sessionId) => {
      id = sessionId;

      return new Promise<ISessionIdentity>((resolve) => (finish = resolve));
    });
    const pending = session.open(command, "project");

    await session.close();

    expect(command).toHaveBeenCalledWith(id, "project");
    expect(release).toHaveBeenCalledWith([id]);

    finish({ sessionId: id });
    await pending;

    expect(release).toHaveBeenCalledTimes(2);
  });

  it("leaves its openings to the scope's release, and closes one published after it", async () => {
    const release = jest.fn<(ids: Array<string>) => Promise<void>>().mockResolvedValue(undefined);
    const scopeService: SessionScopeService = new SessionScopeService("test");
    const session = new Session(release, scopeService);

    await session.open(async (sessionId) => ({ sessionId }));

    let finish!: (value: ISessionIdentity) => void;
    let late: string = "";

    const pending = session.open(async (sessionId) => {
      late = sessionId;

      return new Promise<ISessionIdentity>((resolve) => (finish = resolve));
    });

    // The open reaches its command once its binding is sent.
    await new Promise<void>((resolve) => setTimeout(resolve, 0));
    scopeService.release();
    await session.close();

    expect(release).not.toHaveBeenCalled();

    finish({ sessionId: late });
    await pending;

    expect(release.mock.calls).toEqual([[[late]]]);
  });

  it("does not release a newer opening when an older close finishes", async () => {
    let finishClose!: () => void;

    const release = jest
      .fn<(ids: Array<string>) => Promise<void>>()
      .mockImplementationOnce(() => new Promise<void>((resolve) => (finishClose = resolve)))
      .mockResolvedValue(undefined);

    const session = new Session(release, new SessionScopeService("test"));
    const first = await session.open(async (sessionId) => ({ sessionId }));
    const closing = session.close();
    const next = await session.open(async (sessionId) => ({ sessionId }));

    finishClose();

    await closing;
    await session.close();

    expect(release.mock.calls).toEqual([[[first.sessionId]], [[next.sessionId]]]);
  });

  it("retains the committed session when a replacement fails", async () => {
    const release = jest.fn<(ids: Array<string>) => Promise<void>>().mockResolvedValue(undefined);
    const session = new Session(release, new SessionScopeService("test"));
    const first = await session.open(async (sessionId) => ({ sessionId }));

    await expect(
      session.open(async () => {
        throw new Error("cannot read");
      })
    ).rejects.toThrow("cannot read");

    await session.close();

    expect(release).toHaveBeenCalledWith([first.sessionId]);
  });

  it("releases an adopted opening as its own, and ignores an absent one", async () => {
    const release = jest.fn<(ids: Array<string>) => Promise<void>>().mockResolvedValue(undefined);
    const session = new Session(release, new SessionScopeService("test"));

    session.adopt(null);
    session.adopt(undefined);

    await session.close();

    expect(release).not.toHaveBeenCalled();

    session.adopt({ sessionId: "restored" });

    await session.close();

    expect(release).toHaveBeenCalledWith(["restored"]);
  });

  // The backend holds each opening apart, so one replaced and only forgotten would stay open until its tool is left.
  it("closes an opening, adopted or opened, once a later open replaces it", async () => {
    const release = jest.fn<(ids: Array<string>) => Promise<void>>().mockResolvedValue(undefined);
    const session = new Session(release, new SessionScopeService("test"));

    session.adopt({ sessionId: "restored" });

    const first = await session.open(async (sessionId) => ({ sessionId }));
    const second = await session.open(async (sessionId) => ({ sessionId }));

    await session.close();

    expect(release.mock.calls).toEqual([[["restored"]], [[first.sessionId]], [[second.sessionId]]]);
  });

  it("retries a replaced opening on the next close where closing it failed", async () => {
    const release = jest
      .fn<(ids: Array<string>) => Promise<void>>()
      .mockRejectedValueOnce(new Error("transport unavailable"))
      .mockResolvedValue(undefined);
    const session = new Session(release, new SessionScopeService("test"));

    jest.spyOn(Logger, "error").mockImplementation(() => undefined);
    session.adopt({ sessionId: "restored" });

    const opened = await session.open(async (sessionId) => ({ sessionId }));

    await Promise.resolve();
    await session.close();

    expect(release.mock.calls).toEqual([[["restored"]], [[opened.sessionId, "restored"]]]);
  });

  it("can retry a failed release and includes the adopted session", async () => {
    const release = jest
      .fn<(ids: Array<string>) => Promise<void>>()
      .mockRejectedValueOnce(new Error("transport unavailable"))
      .mockResolvedValue(undefined);

    const session = new Session(release, new SessionScopeService("test"));

    session.adopt({ sessionId: "restored" });

    await expect(session.close()).rejects.toThrow("transport unavailable");
    await session.close();

    expect(release.mock.calls).toEqual([[["restored"]], [["restored"]]]);
  });

  it("swallows a teardown failure, and retries it on the next close", async () => {
    const release = jest
      .fn<(ids: Array<string>) => Promise<void>>()
      .mockRejectedValueOnce(new Error("transport unavailable"))
      .mockResolvedValue(undefined);

    const session = new Session(release, new SessionScopeService("test"));

    session.adopt({ sessionId: "restored" });

    // Nothing to reject into: deactivation has already returned by the time the release settles.
    expect(() => session.release()).not.toThrow();

    await Promise.resolve();
    await session.close();

    expect(release.mock.calls).toEqual([[["restored"]], [["restored"]]]);
  });

  it("releases nothing outside the desktop shell, where there is no backend to ask", async () => {
    const release = jest.fn<(ids: Array<string>) => Promise<void>>().mockResolvedValue(undefined);
    const session = new Session(release, new SessionScopeService("test"));

    mockIsTauri.mockReturnValue(false);
    session.adopt({ sessionId: "restored" });
    session.release();

    await Promise.resolve();

    expect(release).not.toHaveBeenCalled();
  });

  it("retains a late publication when releasing it fails", async () => {
    const release = jest
      .fn<(ids: Array<string>) => Promise<void>>()
      .mockResolvedValueOnce(undefined)
      .mockRejectedValueOnce(new Error("transport unavailable"))
      .mockResolvedValue(undefined);

    const session = new Session(release, new SessionScopeService("test"));

    let finish!: () => void;
    let id: string = "";

    const opening = session.open((sessionId) => {
      id = sessionId;

      return new Promise<ISessionIdentity>((resolve) => (finish = () => resolve({ sessionId })));
    });

    await session.close();

    finish();

    await expect(opening).rejects.toThrow("transport unavailable");
    await session.close();

    expect(release.mock.calls).toEqual([[[id]], [[id]], [[id]]]);
  });

  it("binds an opening to its tool's scope before the open is sent", async () => {
    const release = jest.fn<(ids: Array<string>) => Promise<void>>().mockResolvedValue(undefined);
    const session = new Session(release, new SessionScopeService("level-viewer"));
    const sent: Array<string> = [];

    mockInvoke.mockClear();

    await session.open(async (sessionId: string) => {
      sent.push(...mockInvoke.mock.calls.map(([command]) => command));

      return { sessionId };
    });

    expect(sent).toEqual([ESessionsCommand.BIND]);
    expect(mockInvoke.mock.calls[0]?.[1]).toEqual({ sessionId: expect.any(String), tool: "level-viewer" });
    expect(session.tool).toBe("level-viewer");
  });
});
