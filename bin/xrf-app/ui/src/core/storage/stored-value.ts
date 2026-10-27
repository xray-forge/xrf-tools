import { BoundAction, Computed, makeObservable, RefObservable } from "@wirestate/mobx";
import { Logger } from "@xrf/logging";

import { getLocalStorageValue, setLocalStorageValueSafe } from "@/core/storage/read-write";
import { IStoredValueCodec } from "@/core/storage/stored-value-codec";

/**
 * A preference kept in local storage, observable while the application runs.
 *
 * Read once, when built; each `set` writes through, a refused write logged rather than thrown, as nothing a preference
 * is set from can do about a full quota. A stored value that does not read is dropped for the default, never migrated.
 */
export class StoredValue<T> {
  /**
   * Reads what a key holds as a stored value does when built, for `useStoredValue`, which holds it as React state.
   *
   * @param key - Key from `keys.ts` the value is kept under.
   * @param codec - How the value is kept as a string.
   * @returns The value the codec reads, or its default where what is stored does not read.
   */
  public static read<T>(key: string, codec: IStoredValueCodec<T>): T {
    try {
      return codec.read(getLocalStorageValue(key));
    } catch (error: unknown) {
      Logger.warn("Discarding unreadable local storage value:", key, error);

      return codec.read(null);
    }
  }

  @RefObservable()
  private current: T;

  /**
   * @param key - Key from `keys.ts` the value is kept under.
   * @param codec - How the value is kept as a string.
   */
  public constructor(
    private readonly key: string,
    private readonly codec: IStoredValueCodec<T>
  ) {
    this.current = StoredValue.read(key, codec);

    // Composed state is not activated by the container's observable plugin.
    makeObservable(this);
  }

  @Computed()
  public get value(): T {
    return this.current;
  }

  /**
   * @param value - Value to hold and keep from now on.
   */
  @BoundAction()
  public set(value: T): void {
    this.current = value;
    setLocalStorageValueSafe(this.key, this.codec.write(value));
  }
}
