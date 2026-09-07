import { useCallback, useMemo, useRef, useState } from "react";

import {
  forgetRecentPath,
  IPathRecord,
  readRecentPaths,
  recordRecentPath,
  writeRecentPaths,
} from "@/core/ui/form/path-recents";

/** The paths one field was given before, and the two ways that list changes. */
export interface IPathRecents {
  /** The remembered paths, newest first. */
  records: ReadonlyArray<IPathRecord>;
  /** Puts a path at the head, moving it rather than repeating it. */
  record: (path: string) => void;
  /** Drops one path. */
  forget: (path: string) => void;
}

/**
 * The paths one field was given before, newest first.
 *
 * Read once, on the first render, like every other remembered form value: nothing about mounting or remounting
 * writes, so no session can overwrite what an earlier one recorded while its own list is still empty.
 *
 * Holds only what was recorded. Nothing stands in for an empty history - not the value the field is currently
 * holding, and not the guess a seed produced - because an entry a person never chose or ran is one they cannot
 * account for, and it appears for a field with a stored value while an identical-looking seeded field shows nothing.
 *
 * @param storageKey - Storage key holding the history.
 * @returns The history and the two ways it changes, stable while the list is unchanged.
 */
export function usePathRecents(storageKey: string): IPathRecents {
  const [records, setRecords] = useState<ReadonlyArray<IPathRecord>>(() => readRecentPaths(storageKey));

  // Held as well as rendered, so both actions stay stable while reading the list they are changing.
  const recordsRef = useRef<ReadonlyArray<IPathRecord>>(records);

  recordsRef.current = records;

  const apply = useCallback(
    (next: Array<IPathRecord>): void => {
      setRecords(next);
      writeRecentPaths(storageKey, next);
    },
    [storageKey]
  );

  const record = useCallback(
    (path: string): void => apply(recordRecentPath(recordsRef.current, path, Date.now())),
    [apply]
  );

  const forget = useCallback((path: string): void => apply(forgetRecentPath(recordsRef.current, path)), [apply]);

  // Held together rather than handed out loose: every layer between here and the menu passes the history on whole, so
  // it is one prop with one name the whole way down instead of three that have to stay in step.
  return useMemo(() => ({ forget, record, records }), [forget, record, records]);
}
