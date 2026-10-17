import { Nullable } from "@xrf/types";
import { useCallback, useEffect, useRef, useState } from "react";

/** How often a control dragged by hand tells its owner, in milliseconds: about every third frame, which reads as live. */
export const DEFAULT_DRAFT_INTERVAL: number = 50;

/** What a control shows as it is dragged, and how it reports the drag. */
export interface IThrottledValue<T> {
  /** The value to show: the one being dragged to, or the owner's. */
  value: T;
  /** Whether a drag's value is shown over the owner's. */
  isDrafted: boolean;
  /** A value dragged to, which the owner is told at most once an interval, the latest last. */
  change: (value: T) => void;
  /**
   * The gesture ended: what it dragged to is told at once, or, for one discrete step (`isStep`) with a telling pending,
   * when its interval is up.
   */
  end: (isStep?: boolean) => void;
}

/**
 * A value that follows the hand at once but tells its owner at most once an interval, and always the last.
 *
 * Only what this gesture dragged to is told, never what the owner holds or was last told; the owner holding anything
 * else drops the draft, which also ends a gesture the browser cancelled.
 *
 * @param value - The owner's value.
 * @param isSame - Whether two values are one position of the control, a value told before against one held.
 * @param onChange - Told what the value became.
 * @param intervalMs - The least time between two tellings.
 * @returns What to show, and how to report the drag.
 */
export function useThrottledValue<T>(
  value: T,
  isSame: (left: T, right: Nullable<T>) => boolean,
  onChange: (value: T) => void,
  intervalMs: number = DEFAULT_DRAFT_INTERVAL
): IThrottledValue<T> {
  const [draft, setDraft] = useState<Nullable<{ value: T }>>(null);
  const valueRef = useRef<T>(value);
  const onChangeRef = useRef<(value: T) => void>(onChange);
  const isSameRef = useRef<(left: T, right: Nullable<T>) => boolean>(isSame);
  const pending = useRef<Nullable<{ value: T }>>(null);
  const sent = useRef<Nullable<T>>(null);
  const sentAt = useRef<number>(-Infinity);
  const isEnding = useRef<boolean>(false);
  const timer = useRef<Nullable<ReturnType<typeof setTimeout>>>(null);

  valueRef.current = value;
  onChangeRef.current = onChange;
  isSameRef.current = isSame;

  const cancel = useCallback((): void => {
    if (timer.current !== null) {
      clearTimeout(timer.current);
      timer.current = null;
    }
  }, []);

  const flush = useCallback((): void => {
    timer.current = null;

    const next: Nullable<{ value: T }> = pending.current;

    pending.current = null;

    if (
      next !== null &&
      !isSameRef.current(next.value, sent.current) &&
      !isSameRef.current(next.value, valueRef.current)
    ) {
      sent.current = next.value;
      sentAt.current = performance.now();
      onChangeRef.current(next.value);
    }

    if (isEnding.current) {
      isEnding.current = false;
      setDraft(null);
    }
  }, []);

  const change = useCallback(
    (next: T): void => {
      setDraft({ value: next });
      pending.current = { value: next };
      isEnding.current = false;

      if (timer.current !== null) {
        return;
      }

      const wait: number = sentAt.current + intervalMs - performance.now();

      if (wait <= 0) {
        flush();
      } else {
        timer.current = setTimeout(flush, wait);
      }
    },
    [intervalMs, flush]
  );

  const end = useCallback(
    (isStep: boolean = false): void => {
      isEnding.current = true;

      // A step waits for its interval like any change; letting go tells at once.
      if (timer.current === null || !isStep) {
        cancel();
        flush();
      }
    },
    [cancel, flush]
  );

  // The owner holding something this hook did not send overrides whatever was being dragged.
  useEffect(() => {
    if (!isSameRef.current(value, sent.current)) {
      cancel();
      pending.current = null;
      sent.current = null;
      isEnding.current = false;
      setDraft(null);
    }
  }, [value, cancel]);

  // A control taken down mid-drag still tells what it was dragged to.
  useEffect(
    () => () => {
      if (timer.current !== null) {
        cancel();
        flush();
      }
    },
    [cancel, flush]
  );

  return { change, end, isDrafted: draft !== null, value: draft === null ? value : draft.value };
}
