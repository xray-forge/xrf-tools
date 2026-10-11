import { beforeEach, describe, expect, it } from "@jest/globals";

import {
  clearLocalStorage,
  getLocalStorageValue,
  listLocalStorageKeys,
  setLocalStorageValue,
} from "@/core/storage/read-write";
import { getLocalStorageRevision } from "@/core/storage/revision";

describe("local storage reads and writes", () => {
  beforeEach(() => {
    clearLocalStorage();
  });

  it("lists every key stored, the application's or not", () => {
    setLocalStorageValue("xrf.form.a.b", "C:\\gamedata");
    setLocalStorageValue("something-nobody-grouped", "1");

    expect(listLocalStorageKeys().sort()).toEqual(["something-nobody-grouped", "xrf.form.a.b"]);
  });

  it("clears every key, advancing the revision so readers recompute", () => {
    setLocalStorageValue("xrf.form.a.b", "C:\\gamedata");

    const before: number = getLocalStorageRevision();

    clearLocalStorage();

    expect(listLocalStorageKeys()).toEqual([]);
    expect(getLocalStorageValue("xrf.form.a.b")).toBeNull();
    expect(getLocalStorageRevision()).toBeGreaterThan(before);
  });
});
