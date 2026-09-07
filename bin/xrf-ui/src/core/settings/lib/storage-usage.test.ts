import { beforeEach, describe, expect, it } from "@jest/globals";

import {
  clearStorageGroup,
  describeKeyCount,
  EStorageGroup,
  groupStorageEntries,
  IStorageEntry,
  IStorageGroupUsage,
  IStorageUsage,
  measureLocalStorage,
  STORAGE_GROUPS,
} from "@/core/settings/lib/storage-usage";
import { Optional } from "@/lib/types/general";

/** The measured group with this identity, so a test says which group it means rather than where it sits. */
function findGroup(usage: IStorageUsage, id: EStorageGroup): IStorageGroupUsage {
  const found: Optional<IStorageGroupUsage> = usage.groups.find((it) => it.descriptor.id === id);

  if (!found) {
    throw new Error(`No group ${id}`);
  }

  return found;
}

/** The keys of a measured group, in the order it lists them. */
function keysOf(usage: IStorageGroupUsage): Array<string> {
  return usage.entries.map((it: IStorageEntry) => it.key);
}

describe("measureLocalStorage", () => {
  beforeEach(() => {
    window.localStorage.clear();
  });

  it("reads an empty store as nothing, still naming every group", () => {
    const usage: IStorageUsage = measureLocalStorage();

    expect(usage.total).toBe(0);
    expect(usage.groups).toHaveLength(STORAGE_GROUPS.length);
  });

  it("counts the key as well as the value, two bytes per code unit", () => {
    window.localStorage.setItem("ab", "cde");

    // Both are stored, so both are measured: five code units, ten bytes.
    expect(measureLocalStorage().total).toBe(10);
  });

  it("puts every key in exactly one group, so the parts sum to the whole", () => {
    window.localStorage.setItem("xrf.form-recents.configs-verifier.directory", "[]");
    window.localStorage.setItem("xrf.form.configs-verifier.directory", "C:\\gamedata\\configs");
    window.localStorage.setItem("xrf-gamedata-path", "C:\\gamedata");
    window.localStorage.setItem("xrf.panels.left.width", "320");
    window.localStorage.setItem("xrf-dev-mode", "true");
    window.localStorage.setItem("theme", "dark");
    window.localStorage.setItem("something-nobody-grouped", "1");

    const usage: IStorageUsage = measureLocalStorage();
    const counted: number = usage.groups.reduce((total: number, it: IStorageGroupUsage) => total + it.size, 0);
    const keys: number = usage.groups.reduce((total: number, it: IStorageGroupUsage) => total + it.entries.length, 0);

    expect(counted).toBe(usage.total);
    expect(keys).toBe(window.localStorage.length);
  });

  it("classifies each kind of key where it belongs", () => {
    window.localStorage.setItem("xrf.form-recents.configs-verifier.directory", "[]");
    window.localStorage.setItem("xrf.form.configs-verifier.directory", "C:\\gamedata\\configs");
    window.localStorage.setItem("xrf-gamedata-path", "C:\\gamedata");
    window.localStorage.setItem("xrf.panels.left.width", "320");
    window.localStorage.setItem("theme", "dark");

    const usage: IStorageUsage = measureLocalStorage();

    expect(keysOf(findGroup(usage, EStorageGroup.RECENT_PATHS))).toEqual([
      "xrf.form-recents.configs-verifier.directory",
    ]);
    expect(keysOf(findGroup(usage, EStorageGroup.FORM_VALUES))).toEqual(["xrf.form.configs-verifier.directory"]);
    expect(keysOf(findGroup(usage, EStorageGroup.LAYOUT))).toEqual(["xrf.panels.left.width"]);
    expect(keysOf(findGroup(usage, EStorageGroup.PREFERENCES))).toEqual(["theme"]);
    // A key from a retired feature has no group of its own any more, which is what the catch-all is for.
    expect(keysOf(findGroup(usage, EStorageGroup.OTHER))).toEqual(["xrf-gamedata-path"]);
  });

  it("gives an unclaimed key to the catch-all rather than dropping it", () => {
    window.localStorage.setItem("a-key-from-some-future-feature", "value");

    const usage: IStorageUsage = measureLocalStorage();

    expect(keysOf(findGroup(usage, EStorageGroup.OTHER))).toEqual(["a-key-from-some-future-feature"]);
    expect(usage.total).toBeGreaterThan(0);
  });

  it("lists the biggest key of a group first", () => {
    window.localStorage.setItem("xrf.form.a.b", "short");
    window.localStorage.setItem("xrf.form.c.d", "a much longer stored path than the other one");

    expect(keysOf(findGroup(measureLocalStorage(), EStorageGroup.FORM_VALUES))[0]).toBe("xrf.form.c.d");
  });
});

describe("groupStorageEntries", () => {
  it("gives every entry to exactly one group, whatever the key is", () => {
    // The rule that makes the section honest, checked without a browser: the parts sum to the whole because the last
    // group matches everything rather than because it happens to be last.
    const entries: Array<IStorageEntry> = [
      { key: "xrf.form-recents.a.b", size: 10 },
      { key: "xrf.form.a.b", size: 20 },
      { key: "xrf-gamedata-path", size: 40 },
      { key: "xrf.panels.left.width", size: 80 },
      { key: "theme", size: 160 },
      { key: "nothing-claims-this", size: 320 },
    ];

    const usage: IStorageUsage = groupStorageEntries(entries);

    expect(usage.total).toBe(630);
    expect(usage.groups.reduce((total: number, it: IStorageGroupUsage) => total + it.size, 0)).toBe(630);
    expect(usage.groups.reduce((total: number, it: IStorageGroupUsage) => total + it.entries.length, 0)).toBe(
      entries.length
    );
  });

  it("names every group even for no entries at all", () => {
    const usage: IStorageUsage = groupStorageEntries([]);

    expect(usage.total).toBe(0);
    expect(usage.groups.map((it: IStorageGroupUsage) => it.descriptor.id)).toEqual(STORAGE_GROUPS.map((it) => it.id));
  });
});

describe("describeKeyCount", () => {
  it("agrees with its noun", () => {
    expect(describeKeyCount(0)).toBe("0 keys");
    expect(describeKeyCount(1)).toBe("1 key");
    expect(describeKeyCount(2)).toBe("2 keys");
  });
});

describe("clearStorageGroup", () => {
  beforeEach(() => {
    window.localStorage.clear();
  });

  it("empties its own keys and leaves every other group alone", () => {
    window.localStorage.setItem("xrf.form-recents.a.b", "[]");
    window.localStorage.setItem("xrf.form.a.b", "C:\\gamedata");
    window.localStorage.setItem("xrf-gamedata-path", "C:\\gamedata");

    const usage: IStorageUsage = measureLocalStorage();

    clearStorageGroup(findGroup(usage, EStorageGroup.RECENT_PATHS));

    expect(window.localStorage.getItem("xrf.form-recents.a.b")).toBeNull();
    expect(window.localStorage.getItem("xrf.form.a.b")).toBe("C:\\gamedata");
    expect(window.localStorage.getItem("xrf-gamedata-path")).toBe("C:\\gamedata");
  });
});
