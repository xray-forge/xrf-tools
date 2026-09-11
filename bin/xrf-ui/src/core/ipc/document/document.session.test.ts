import { describe, expect, it, jest } from "@jest/globals";

import { DocumentSession, IDocumentSession } from "./index";

describe("DocumentSession", () => {
  it("releases an opening before its response and releases a late publication again", async () => {
    const release = jest.fn<(ids: Array<string>) => Promise<void>>().mockResolvedValue(undefined);
    const session = new DocumentSession(release);

    let finish!: (value: IDocumentSession) => void;
    let id: string = "";

    const pending = session.open((sessionId) => {
      id = sessionId;

      return new Promise<IDocumentSession>((resolve) => (finish = resolve));
    });

    await session.close();

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

    const session = new DocumentSession(release);
    const first = await session.open(async (sessionId) => ({ sessionId }));
    const closing = session.close();
    const next = await session.open(async (sessionId) => ({ sessionId }));

    finishClose();

    await closing;
    await session.close();

    expect(release.mock.calls).toEqual([[[first.sessionId]], [[next.sessionId]]]);
  });

  it("retains the committed document when a replacement fails", async () => {
    const release = jest.fn<(ids: Array<string>) => Promise<void>>().mockResolvedValue(undefined);
    const session = new DocumentSession(release);
    const first = await session.open(async (sessionId) => ({ sessionId }));

    await expect(
      session.open(async () => {
        throw new Error("cannot read");
      })
    ).rejects.toThrow("cannot read");

    await session.close();

    expect(release).toHaveBeenCalledWith([first.sessionId]);
  });

  it("can retry a failed release and includes the restored document", async () => {
    const release = jest
      .fn<(ids: Array<string>) => Promise<void>>()
      .mockRejectedValueOnce(new Error("transport unavailable"))
      .mockResolvedValue(undefined);

    const session = new DocumentSession(release);

    await expect(session.close("restored")).rejects.toThrow("transport unavailable");
    await session.close();

    expect(release.mock.calls).toEqual([[["restored"]], [["restored"]]]);
  });

  it("retains a late publication when releasing it fails", async () => {
    const release = jest
      .fn<(ids: Array<string>) => Promise<void>>()
      .mockResolvedValueOnce(undefined)
      .mockRejectedValueOnce(new Error("transport unavailable"))
      .mockResolvedValue(undefined);

    const session = new DocumentSession(release);

    let finish!: () => void;
    let id: string = "";

    const opening = session.open((sessionId) => {
      id = sessionId;

      return new Promise<IDocumentSession>((resolve) => (finish = () => resolve({ sessionId })));
    });

    await session.close();

    finish();

    await expect(opening).rejects.toThrow("transport unavailable");
    await session.close();

    expect(release.mock.calls).toEqual([[[id]], [[id]], [[id]]]);
  });
});
