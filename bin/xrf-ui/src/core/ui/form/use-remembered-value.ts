import { useCallback, useState } from "react";

import { EApplicationId } from "@/core/routing/application";
import { getLocalStorageValue, setLocalStorageValue } from "@/lib/local-storage";
import { Nullable } from "@/lib/types/general";

const STORAGE_PREFIX: string = "xrf.form.";

export interface IRememberedValueOptions<T extends string> {
  /** The application that owns this field. */
  application: EApplicationId;
  /** Names the field inside its own form, for example `language`. Only unique within the application. */
  id: string;
  /** Used when nothing was remembered, and when what was remembered is no longer offered. */
  fallback: T;
  /** What the field currently accepts, so a stale stored value cannot survive a change to the options. */
  allowed: ReadonlyArray<T>;
}

/**
 * A form value that is remembered between sessions, under the key scheme paths already use.
 *
 * @param options - Field identity, its fallback, and the values it accepts.
 * @param options.application - Application the field belongs to, used to scope persistence.
 * @param options.id - Field name within that application, used for persistence.
 * @param options.fallback - Value used when nothing valid was remembered.
 * @param options.allowed - The values this field accepts.
 * @returns The current value and a setter that also records it.
 */
export function useRememberedValue<T extends string>({
  application,
  id,
  fallback,
  allowed,
}: IRememberedValueOptions<T>): [T, (value: T) => void] {
  const storageKey: string = `${STORAGE_PREFIX}${application}.${id}`;

  // Read once, on the first render. Nothing about mounting or remounting writes, so no session can
  // overwrite what an earlier one remembered while its own state is still empty.
  const [value, setValue] = useState<T>(() => {
    const stored: Nullable<string> = getLocalStorageValue(storageKey);

    return stored && allowed.includes(stored as T) ? (stored as T) : fallback;
  });

  const remember = useCallback(
    (next: T): void => {
      setValue(next);
      setLocalStorageValue(storageKey, next);
    },
    [storageKey]
  );

  return [value, remember];
}
