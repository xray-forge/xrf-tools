import { afterEach, beforeEach, describe, expect, it, jest } from "@jest/globals";

import {
  forgetRecentPath,
  IPathRecord,
  readRecentPaths,
  RECENT_PATHS_LIMIT,
  recordRecentPath,
  writeRecentPaths,
} from "@/core/ui/form/path-recents";

describe("recordRecentPath", () => {
  it("puts the newest path first", () => {
    const first: Array<IPathRecord> = recordRecentPath([], "C:\\gamedata", 1);
    const second: Array<IPathRecord> = recordRecentPath(first, "C:\\gamedata-anomaly", 2);

    expect(second.map((it: IPathRecord) => it.path)).toEqual(["C:\\gamedata-anomaly", "C:\\gamedata"]);
  });

  it("moves a path already listed rather than repeating it", () => {
    const listed: Array<IPathRecord> = recordRecentPath(recordRecentPath([], "C:\\a", 1), "C:\\b", 2);
    const again: Array<IPathRecord> = recordRecentPath(listed, "C:\\a", 3);

    expect(again.map((it: IPathRecord) => it.path)).toEqual(["C:\\a", "C:\\b"]);
    expect(again[0].at).toBe(3);
  });

  it("treats the spellings of one windows path as one entry, keeping the one that arrived", () => {
    const listed: Array<IPathRecord> = recordRecentPath([], "C:\\Projects\\Gamedata", 1);
    const again: Array<IPathRecord> = recordRecentPath(listed, "c:/projects/gamedata\\", 2);

    expect(again).toHaveLength(1);
    // Stored verbatim: what is offered has to be what the dialog or the person actually gave.
    expect(again[0].path).toBe("c:/projects/gamedata\\");
  });

  it("drops the oldest once the list is full", () => {
    let records: Array<IPathRecord> = [];

    for (let index = 0; index <= RECENT_PATHS_LIMIT; index += 1) {
      records = recordRecentPath(records, `C:\\path-${index}`, index);
    }

    expect(records).toHaveLength(RECENT_PATHS_LIMIT);
    expect(records[0].path).toBe(`C:\\path-${RECENT_PATHS_LIMIT}`);
    expect(records.map((it: IPathRecord) => it.path)).not.toContain("C:\\path-0");
  });

  it("records nothing for an empty path", () => {
    expect(recordRecentPath([], "", 1)).toEqual([]);
  });
});

describe("forgetRecentPath", () => {
  it("drops the entry whichever way the path is spelled", () => {
    const records: Array<IPathRecord> = recordRecentPath(recordRecentPath([], "C:\\a", 1), "C:\\b", 2);

    expect(forgetRecentPath(records, "c:/a/").map((it: IPathRecord) => it.path)).toEqual(["C:\\b"]);
  });
});

describe("readRecentPaths", () => {
  const KEY: string = "xrf.form-recents.configs-verifier.directory";

  beforeEach(() => {
    window.localStorage.clear();
  });

  it("reads back what was written", () => {
    writeRecentPaths(KEY, [{ at: 7, path: "C:\\gamedata" }]);

    expect(readRecentPaths(KEY)).toEqual([{ at: 7, path: "C:\\gamedata" }]);
  });

  it("reads an absent key as no history at all", () => {
    expect(readRecentPaths(KEY)).toEqual([]);
  });

  it("keeps the sound entries of a list holding one that is not a record", () => {
    // The whole point of the failsafe rule: one bad element must not cost the other nine.
    window.localStorage.setItem(
      KEY,
      JSON.stringify([
        { at: 1, path: "C:\\a" },
        { at: "yesterday", path: "C:\\b" },
        null,
        { path: "C:\\c" },
        42,
        { at: 2, path: "" },
        { at: 3, path: "C:\\d" },
      ])
    );

    expect(readRecentPaths(KEY).map((it: IPathRecord) => it.path)).toEqual(["C:\\a", "C:\\d"]);
  });

  it("reads content that is not a list at all as no history", () => {
    window.localStorage.setItem(KEY, "not json");
    expect(readRecentPaths(KEY)).toEqual([]);

    window.localStorage.setItem(KEY, JSON.stringify({ path: "C:\\a" }));
    expect(readRecentPaths(KEY)).toEqual([]);
  });

  it("collapses duplicates left by something else, so a caller never has to", () => {
    window.localStorage.setItem(
      KEY,
      JSON.stringify([
        { at: 2, path: "C:\\Gamedata" },
        { at: 1, path: "c:/gamedata/" },
      ])
    );

    expect(readRecentPaths(KEY)).toHaveLength(1);
  });

  it("reads at most the limit, however much was stored", () => {
    window.localStorage.setItem(
      KEY,
      JSON.stringify(
        Array.from({ length: RECENT_PATHS_LIMIT + 5 }, (_: unknown, index: number) => ({
          at: index,
          path: `C:\\path-${index}`,
        }))
      )
    );

    expect(readRecentPaths(KEY)).toHaveLength(RECENT_PATHS_LIMIT);
  });
});

describe("writeRecentPaths", () => {
  const KEY: string = "xrf.form-recents.configs-verifier.directory";

  afterEach(() => {
    jest.restoreAllMocks();
    window.localStorage.clear();
  });

  it("removes the key rather than storing an empty list", () => {
    window.localStorage.setItem(KEY, JSON.stringify([{ at: 1, path: "C:\\a" }]));

    writeRecentPaths(KEY, []);

    expect(window.localStorage.getItem(KEY)).toBeNull();
  });

  it("does not raise when storage refuses the write", () => {
    // Spied on the prototype: jsdom's storage is a proxy, so assigning to the instance would store a value under the
    // key "setItem" instead of replacing the method.
    jest.spyOn(Storage.prototype, "setItem").mockImplementation(() => {
      throw new DOMException("exceeded the quota", "QuotaExceededError");
    });

    // A history is a convenience. Whatever asked for it to be recorded has work of its own to finish.
    expect(() => writeRecentPaths(KEY, [{ at: 1, path: "C:\\a" }])).not.toThrow();
  });
});
