import { Nullable } from "@xrf/types";
import { useMemo } from "react";

import { EToolId } from "@/core/routing/tool";
import { getFieldValueStorageKey, IStoredValueCodec, textCodec, useStoredValue } from "@/core/storage";

export interface IRememberedValueOptions<T extends string> {
  /** The tool that owns this field. */
  tool: EToolId;
  /** Names the field inside its own form, for example `language`. Only unique within the tool. */
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
 * @param options.tool - Tool the field belongs to, used to scope persistence.
 * @param options.id - Field name within that tool, used for persistence.
 * @param options.fallback - Value used when nothing valid was remembered.
 * @param options.allowed - The values this field accepts.
 * @returns The current value and a setter that also records it.
 */
export function useRememberedValue<T extends string>({
  tool,
  id,
  fallback,
  allowed,
}: IRememberedValueOptions<T>): [T, (value: T) => void] {
  const codec: IStoredValueCodec<T> = useMemo(
    () => textCodec((stored: Nullable<string>) => (stored && allowed.includes(stored as T) ? (stored as T) : fallback)),
    [allowed, fallback]
  );

  // Read once, on the first render. Nothing about mounting or remounting writes, so no session can overwrite what an
  // earlier one remembered while its own state is still empty.
  return useStoredValue<T>(getFieldValueStorageKey(tool, id), codec);
}
