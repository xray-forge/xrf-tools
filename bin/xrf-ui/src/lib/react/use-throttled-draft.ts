import { Nullable } from "@xrf/types";
import { SyntheticEvent, useCallback } from "react";

import { DEFAULT_DRAFT_INTERVAL, IThrottledValue, useThrottledValue } from "@/lib/react/use-throttled-value";

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
 * A slider value that follows the hand at once but tells its owner at most once an interval, and always the last, as
 * `useThrottledValue` does for any value.
 *
 * @param value - The owner's value.
 * @param step - The slider's step: values within half of one are the same position, since an owner storing a mapping of
 *   the value hands back its round trip, which floats need not return exactly; only equal ones for a step of none.
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
  // A step that is not a positive number has no half to be within: every value would read as an override.
  const tolerance: number = step > 0 && Number.isFinite(step) ? step / 2 : 0;
  const isSame = useCallback(
    (left: number, right: Nullable<number>): boolean =>
      right !== null && (tolerance > 0 ? Math.abs(left - right) < tolerance : Object.is(left, right)),
    [tolerance]
  );
  const draft: IThrottledValue<number> = useThrottledValue(value, isSame, onChange, intervalMs);
  const { change, end } = draft;

  const onSliderChange = useCallback(
    (_: Event, next: TSliderValue): void => change(typeof next === "number" ? next : next[0]),
    [change]
  );
  const onSliderCommitted = useCallback(
    (event: Event | SyntheticEvent): void => end(STEP_COMMIT_EVENTS.has(event.type)),
    [end]
  );

  return { onChange: onSliderChange, onChangeCommitted: onSliderCommitted, value: draft.value };
}
