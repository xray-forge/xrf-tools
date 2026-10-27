import { useCallback, useState } from "react";

import { setLocalStorageValueSafe } from "@/core/storage/read-write";
import { StoredValue } from "@/core/storage/stored-value";
import { IStoredValueCodec } from "@/core/storage/stored-value-codec";

/** A held value with the key it was read under, so a change of key is seen on the render it happens. */
interface IHeldStoredValue<T> {
  key: string;
  value: T;
}

/**
 * A preference kept in local storage, held as a component's state: the hook form of {@link StoredValue}, for a value
 * no service shares.
 *
 * Read on the first render and again on the render the key changes, so a component moved to another key shows that
 * key's value from its first render under it; each set writes through as `StoredValue.set` does.
 *
 * @param key - Key from `keys.ts` the value is kept under.
 * @param codec - How the value is kept as a string; kept stable across renders, as the setter changes with it.
 * @returns The value, and a setter that holds and keeps it.
 */
export function useStoredValue<T>(key: string, codec: IStoredValueCodec<T>): [T, (value: T) => void] {
  const [held, setHeld] = useState<IHeldStoredValue<T>>(() => ({ key, value: StoredValue.read(key, codec) }));

  let current: IHeldStoredValue<T> = held;

  if (held.key !== key) {
    current = { key, value: StoredValue.read(key, codec) };
    setHeld(current);
  }

  const set = useCallback(
    (value: T): void => {
      setHeld({ key, value });
      setLocalStorageValueSafe(key, codec.write(value));
    },
    [codec, key]
  );

  return [current.value, set];
}
