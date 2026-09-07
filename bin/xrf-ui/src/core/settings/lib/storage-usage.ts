import { IWorkspacePathDescriptor, WORKSPACE_PATHS } from "@/core/settings/lib/workspace-path";
import { isFieldRecentsStorageKey, isFieldValueStorageKey } from "@/core/ui/form/field-storage";
import { setLocalStorageValue } from "@/lib/local-storage";
import { BYTES_PER_MEGABYTE } from "@/lib/memory/size";
import { Optional } from "@/lib/types/general";

/**
 * What the application allows itself.
 *
 * Ours rather than the browser's. A per-origin cap exists and is metered in UTF-16 code units, but its exact value is
 * a Chromium internal nobody here has measured in this webview, and a figure shown against a guessed denominator says
 * less than one shown against a stated budget.
 */
export const STORAGE_BUDGET_BYTES: number = 5 * BYTES_PER_MEGABYTE;

/** Bytes one UTF-16 code unit occupies, which is how stored text is counted against a per-origin cap. */
const BYTES_PER_CODE_UNIT: number = 2;

/** What the groups are, in the order the section lists them. */
export enum EStorageGroup {
  RECENT_PATHS = "recentPaths",
  FORM_VALUES = "formValues",
  WORKSPACE_PATHS = "workspacePaths",
  LAYOUT = "layout",
  PREFERENCES = "preferences",
  OTHER = "other",
}

/** One key and what it occupies. */
export interface IStorageEntry {
  key: string;
  /** Bytes, counting the key as well as the value, because both are stored. */
  size: number;
}

/** What one group is, what it holds, and whether a person may empty it here. */
export interface IStorageGroupDescriptor {
  id: EStorageGroup;
  label: string;
  description: string;
  /**
   * Whether this section offers to empty it.
   *
   * False for anything the application needs or another section owns: configured paths are cleared per row in the
   * Paths section, and clearing a preference here would be a second, worse way to change a setting.
   */
  isClearable: boolean;
  /** Whether a key belongs here. Matched in table order, so the catch-all claims what the others declined. */
  matches: (key: string) => boolean;
}

export interface IStorageGroupUsage {
  descriptor: IStorageGroupDescriptor;
  entries: Array<IStorageEntry>;
  size: number;
}

export interface IStorageUsage {
  /** Bytes over every key, so the groups below always sum to this. */
  total: number;
  groups: Array<IStorageGroupUsage>;
}

/** Keys the workspace paths occupy, read from the table that declares them. */
const WORKSPACE_PATH_KEYS: ReadonlyArray<string> = WORKSPACE_PATHS.map(
  (it: IWorkspacePathDescriptor) => it.storageKey
);

/**
 * Keys that are one switch each.
 *
 * Named rather than matched by prefix because they are not spelled alike: three naming styles reached this list, and a
 * table is the honest way to say so.
 */
const PREFERENCE_KEYS: ReadonlyArray<string> = ["theme", "xrf-catalog-view", "xrf-dev-mode", "xrf.media.volume"];

/**
 * Every group, most specific first.
 *
 * The last one matches everything, which is what makes it the catch-all: a key no earlier group claims still has to
 * appear, or the parts stop summing to the whole and the section lies by omission. Said as a predicate rather than
 * left to position, so reordering this table cannot silently move keys into it.
 */
export const STORAGE_GROUPS: ReadonlyArray<IStorageGroupDescriptor> = [
  {
    description: "Paths each picker offers again, ten per field.",
    id: EStorageGroup.RECENT_PATHS,
    isClearable: true,
    label: "Recent paths",
    matches: isFieldRecentsStorageKey,
  },
  {
    description: "The value each form field is holding, restored the next time it opens.",
    id: EStorageGroup.FORM_VALUES,
    isClearable: true,
    label: "Remembered form values",
    matches: isFieldValueStorageKey,
  },
  {
    description: "Game data and its overrides. Cleared one at a time in Paths, because every tool derives from them.",
    id: EStorageGroup.WORKSPACE_PATHS,
    isClearable: false,
    label: "Workspace paths",
    matches: (key: string) => WORKSPACE_PATH_KEYS.includes(key),
  },
  {
    description: "Which side panels are open, and how wide they are.",
    id: EStorageGroup.LAYOUT,
    isClearable: true,
    label: "Layout",
    matches: (key: string) => key.startsWith("xrf.panels."),
  },
  {
    description: "Theme, developer mode, catalog view, playback volume.",
    id: EStorageGroup.PREFERENCES,
    isClearable: false,
    label: "Preferences",
    matches: (key: string) => PREFERENCE_KEYS.includes(key),
  },
  {
    description: "Anything else the application has left here.",
    id: EStorageGroup.OTHER,
    isClearable: false,
    label: "Other",
    matches: () => true,
  },
];

/**
 * What local storage currently holds, by group.
 *
 * @returns The total and every group, largest entries first within each.
 */
export function measureLocalStorage(): IStorageUsage {
  return groupStorageEntries(readStorageEntries());
}

/**
 * Every key currently stored, and what it occupies.
 *
 * The one part that reads the browser. Nothing is parsed as JSON: this measures storage, not meaning.
 *
 * @returns One entry per key, in whatever order storage lists them.
 */
export function readStorageEntries(): Array<IStorageEntry> {
  if (!window.localStorage) {
    return [];
  }

  const entries: Array<IStorageEntry> = [];

  for (let index = 0; index < window.localStorage.length; index += 1) {
    const key: Optional<string> = window.localStorage.key(index) ?? undefined;

    if (key === undefined) {
      continue;
    }

    const characters: number = key.length + (window.localStorage.getItem(key)?.length ?? 0);

    entries.push({ key, size: characters * BYTES_PER_CODE_UNIT });
  }

  return entries;
}

/**
 * Entries folded into {@link STORAGE_GROUPS}, each entry counted exactly once.
 *
 * Separate from reading them so the classification can be checked without a browser, which is where the rule that
 * everything lands somewhere actually lives.
 *
 * @param entries - The entries to fold.
 * @returns The total and every group, largest entries first within each.
 */
export function groupStorageEntries(entries: ReadonlyArray<IStorageEntry>): IStorageUsage {
  const groups: Array<IStorageGroupUsage> = STORAGE_GROUPS.map((descriptor: IStorageGroupDescriptor) => ({
    descriptor,
    entries: [],
    size: 0,
  }));

  let total: number = 0;

  for (const entry of entries) {
    // The catch-all matches everything, so this always finds a group.
    const group: IStorageGroupUsage = groups.find((it) => it.descriptor.matches(entry.key)) as IStorageGroupUsage;

    group.entries.push(entry);
    group.size += entry.size;
    total += entry.size;
  }

  for (const group of groups) {
    group.entries.sort((left: IStorageEntry, right: IStorageEntry) => right.size - left.size);
  }

  return { groups, total };
}

/**
 * A key count a person can read.
 *
 * @param count - How many keys there are.
 * @returns The count with its noun agreeing.
 */
export function describeKeyCount(count: number): string {
  return count === 1 ? "1 key" : `${count} keys`;
}

/**
 * Empties one group.
 *
 * Takes the measured group rather than its descriptor, so what is removed is exactly what was counted and shown.
 * Goes through the ordinary write, one key at a time, so watchers hear about it and no other group's keys are touched.
 *
 * @param group - The group to empty.
 */
export function clearStorageGroup(group: IStorageGroupUsage): void {
  for (const entry of group.entries) {
    setLocalStorageValue(entry.key, null);
  }
}
