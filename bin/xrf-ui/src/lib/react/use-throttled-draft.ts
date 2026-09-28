import { Nullable } from "@xrf/types";
import { SyntheticEvent, useCallback, useEffect, useRef, useState } from "react";

/** How often a control dragged by hand tells its owner, in milliseconds: about every third frame, which reads as live. */
export const DEFAULT_DRAFT_INTERVAL: number = 50;

/** Commits a slider reports for one discrete step rather than for letting go: a key, or its hidden input changing. */
const STEP_COMMIT_EVENTS: ReadonlySet<string> = new Set(["keydown", "change"]);

/** What a slider reports its value as: one thumb, or several. */
export type TSliderValue = number | ReadonlyArray<number>;

/** What a slider shows as it is dragged, and the handlers a MUI `Slider` takes to say so. */
export interface IThrottledDraft {
  /** The value to show: the one being dragged to, or the owner's. */
  value: number;
  /** A value dragged to, which the owner is told at most once an interval, the latest last. */
  onChange: (event: Event, value: TSliderValue) => void;
  /** The gesture ended: what it dragged to is told at once. The value passed is ignored, since MUI's can be stale. */
  onChangeCommitted: (event: Event | SyntheticEvent, value: TSliderValue) => void;
}

/**
 * A slider value that follows the hand at once but tells its owner at most once an interval, and always the last.
 *
 * Only what this gesture dragged to is told, never what the owner holds or was last told; the owner holding anything
 * else drops the draft, which also ends a gesture the browser cancelled.
 *
 * @param value - The owner's value.
 * @param step - The slider's step: values within half of one are the same position, since an owner storing a mapping of
 *   the value hands back its round trip, which floats need not return exactly.
 * @param onChange - Told what the value became.
 * @param intervalMs - The least time between two tellings.
 * @returns What to show, and the slider's `onChange` and `onChangeCommitted`.
 */
export function useThrottledDraft(
  value: number,
  step: number,
  onChange: (value: number) => void,
  intervalMs: number = DEFAULT_DRAFT_INTERVAL
): IThrottledDraft {
  const [draft, setDraft] = useState<Nullable<number>>(null);
  const valueRef = useRef<number>(value);
  const onChangeRef = useRef<(value: number) => void>(onChange);
  const pending = useRef<Nullable<number>>(null);
  const sent = useRef<Nullable<number>>(null);
  const sentAt = useRef<number>(-Infinity);
  const isEnding = useRef<boolean>(false);
  const timer = useRef<Nullable<ReturnType<typeof setTimeout>>>(null);

  valueRef.current = value;
  onChangeRef.current = onChange;

  const isSame = useCallback(
    (left: number, right: Nullable<number>): boolean => right !== null && Math.abs(left - right) < step / 2,
    [step]
  );

  const cancel = useCallback((): void => {
    if (timer.current !== null) {
      clearTimeout(timer.current);
      timer.current = null;
    }
  }, []);

  const flush = useCallback((): void => {
    timer.current = null;

    const next: Nullable<number> = pending.current;

    pending.current = null;

    if (next !== null && !isSame(next, sent.current) && !isSame(next, valueRef.current)) {
      sent.current = next;
      sentAt.current = performance.now();
      onChangeRef.current(next);
    }

    if (isEnding.current) {
      isEnding.current = false;
      setDraft(null);
    }
  }, [isSame]);

  const change = useCallback(
    (_: Event, next: TSliderValue): void => {
      const scalar: number = typeof next === "number" ? next : next[0];

      setDraft(scalar);
      pending.current = scalar;
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

  const commit = useCallback(
    (event: Event | SyntheticEvent): void => {
      isEnding.current = true;

      // A step waits for its interval like any change; letting go tells at once.
      if (timer.current === null || !STEP_COMMIT_EVENTS.has(event.type)) {
        cancel();
        flush();
      }
    },
    [cancel, flush]
  );

  // The owner holding something this hook did not send overrides whatever was being dragged.
  useEffect(() => {
    if (!isSame(value, sent.current)) {
      cancel();
      pending.current = null;
      sent.current = null;
      isEnding.current = false;
      setDraft(null);
    }
  }, [value, cancel, isSame]);

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

  return { onChange: change, onChangeCommitted: commit, value: draft === null ? value : draft };
}
