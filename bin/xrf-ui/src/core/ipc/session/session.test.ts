import { describe, expect, it, jest } from "@jest/globals";

import { ISessionIdentity, Session } from "./index";

describe("Session", () => {
  it("releases an opening before its response and releases a late publication again", async () => {
    const release = jest.fn<(ids: Array<string>) => Promise<void>>().mockResolvedValue(undefined);
    const session = new Session(release);

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

  it("does not release a newer opening when an older close finishes", async () => {
    let finishClose!: () => void;

    const release = jest
      .fn<(ids: Array<string>) => Promise<void>>()
      .mockImplementationOnce(() => new Promise<void>((resolve) => (finishClose = resolve)))
      .mockResolvedValue(undefined);

    const session = new Session(release);
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
    const session = new Session(release);
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
    const session = new Session(release);

    session.adopt(null);
    session.adopt(undefined);

    await session.close();

    expect(release).not.toHaveBeenCalled();

    session.adopt({ sessionId: "restored" });

    await session.close();

    expect(release).toHaveBeenCalledWith(["restored"]);
  });

  it("drops an adopted opening once a later open replaces it", async () => {
    const release = jest.fn<(ids: Array<string>) => Promise<void>>().mockResolvedValue(undefined);
    const session = new Session(release);

    session.adopt({ sessionId: "restored" });

    const opened = await session.open(async (sessionId) => ({ sessionId }));

    await session.close();

    expect(release.mock.calls).toEqual([[[opened.sessionId]]]);
  });

  it("can retry a failed release and includes the adopted session", async () => {
    const release = jest
      .fn<(ids: Array<string>) => Promise<void>>()
      .mockRejectedValueOnce(new Error("transport unavailable"))
      .mockResolvedValue(undefined);

    const session = new Session(release);

    session.adopt({ sessionId: "restored" });

    await expect(session.close()).rejects.toThrow("transport unavailable");
    await session.close();

    expect(release.mock.calls).toEqual([[["restored"]], [["restored"]]]);
  });

  it("retains a late publication when releasing it fails", async () => {
    const release = jest
      .fn<(ids: Array<string>) => Promise<void>>()
      .mockResolvedValueOnce(undefined)
      .mockRejectedValueOnce(new Error("transport unavailable"))
      .mockResolvedValue(undefined);

    const session = new Session(release);

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
});
