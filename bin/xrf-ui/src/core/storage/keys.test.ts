import { readFileSync } from "fs";
import { resolve } from "path";

import { describe, expect, it } from "@jest/globals";

import {
  CATALOG_VIEW_STORAGE_KEY,
  DEV_MODE_STORAGE_KEY,
  getFieldRecentsStorageKey,
  getFieldValueStorageKey,
  getPanelSelectionStorageKey,
  getPanelWidthStorageKey,
  IPC_PROFILING_STORAGE_KEY,
  MEDIA_VOLUME_STORAGE_KEY,
  THEME_STORAGE_KEY,
} from "@/core/storage/keys";
import { EStorageNamespace, isInStorageNamespace } from "@/core/storage/namespace";

/** Every key this application writes that is not built from a caller's own words. */
const FIXED_KEYS: ReadonlyArray<string> = [
  CATALOG_VIEW_STORAGE_KEY,
  DEV_MODE_STORAGE_KEY,
  IPC_PROFILING_STORAGE_KEY,
  MEDIA_VOLUME_STORAGE_KEY,
  THEME_STORAGE_KEY,
];

/** Every key, including one built example per builder, which is what a rule about spelling has to cover. */
const ALL_KEYS: ReadonlyArray<string> = [
  ...FIXED_KEYS,
  getFieldValueStorageKey("configs-verifier", "directory"),
  getFieldRecentsStorageKey("configs-verifier", "directory"),
  getPanelWidthStorageKey("left"),
  getPanelSelectionStorageKey("left", "configs-explorer"),
];

describe("storage keys", () => {
  it("spells every key the one way, because a second spelling is how the space drifted before", () => {
    for (const key of ALL_KEYS) {
      expect(key).toMatch(/^xrf(\.[a-z0-9]+(-[a-z0-9]+)*)+$/);
    }
  });

  it("puts every key in exactly one namespace, which is what lets the Settings section sum to the whole", () => {
    const namespaces: Array<EStorageNamespace> = Object.values(EStorageNamespace);

    for (const key of ALL_KEYS) {
      expect(namespaces.filter((namespace) => isInStorageNamespace(key, namespace))).toHaveLength(1);
    }
  });

  it("keeps a field's value and its history apart, so clearing one cannot reach the other", () => {
    const value: string = getFieldValueStorageKey("configs-verifier", "directory");
    const recents: string = getFieldRecentsStorageKey("configs-verifier", "directory");

    expect(value).not.toBe(recents);
    expect(isInStorageNamespace(value, EStorageNamespace.FORM_RECENTS)).toBe(false);
    expect(isInStorageNamespace(recents, EStorageNamespace.FORM)).toBe(false);
  });

  it("names each fixed key once, because two switches sharing a key is one switch", () => {
    expect(new Set(FIXED_KEYS).size).toBe(FIXED_KEYS.length);
  });

  it("is repeated correctly by the one script that cannot import it", () => {
    // `theme-init.ts` is inlined into `index.html` and runs before any module graph exists, so it spells the theme key
    // out. Read rather than imported, because the script is an IIFE with nothing to import and a drift between the two
    // spellings shows up as a page that paints in the wrong scheme - a thing no other test would notice.
    const script: string = readFileSync(resolve(__dirname, "../../theme-init.ts"), "utf8");

    expect(script).toContain(`"${THEME_STORAGE_KEY}"`);
  });
});
