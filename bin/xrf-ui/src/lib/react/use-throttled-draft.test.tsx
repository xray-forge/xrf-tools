import { afterEach, beforeEach, describe, expect, it, jest } from "@jest/globals";
import { act, renderHook } from "@testing-library/react";

import { useThrottledDraft } from "@/lib/react/use-throttled-draft";

const MOVE: Event = new Event("pointermove");
const RELEASE: Event = new Event("pointerup");
const KEY: Event = new Event("keydown");

describe("useThrottledDraft", () => {
  beforeEach(() => {
    jest.useFakeTimers();
  });

  afterEach(() => {
    jest.useRealTimers();
  });

  it("shows every value dragged to, and tells the owner at most once an interval, the latest last", () => {
    const onChange = jest.fn();
    const { result } = renderHook(() => useThrottledDraft(0, 1, onChange, 200));

    act(() => result.current.onChange(MOVE, 1));
    act(() => result.current.onChange(MOVE, 2));
    act(() => result.current.onChange(MOVE, [3]));

    expect(result.current.value).toBe(3);
    expect(onChange.mock.calls).toEqual([[1]]);

    act(() => jest.advanceTimersByTime(200));

    expect(onChange.mock.calls).toEqual([[1], [3]]);
  });

  it("tells the value let go at at once, and falls back to the owner's after", () => {
    const onChange = jest.fn();
    const { result, rerender } = renderHook(({ value }) => useThrottledDraft(value, 1, onChange, 200), {
      initialProps: { value: 0 },
    });

    act(() => result.current.onChange(MOVE, 1));
    act(() => result.current.onChange(MOVE, 2));
    act(() => result.current.onChangeCommitted(RELEASE, 2));
    act(() => jest.advanceTimersByTime(1000));

    expect(onChange.mock.calls).toEqual([[1], [2]]);

    rerender({ value: 5 });

    expect(result.current.value).toBe(5);
  });

  it("tells nothing twice when the value let go at is the one already told", () => {
    const onChange = jest.fn();
    const { result } = renderHook(() => useThrottledDraft(0, 1, onChange, 200));

    act(() => result.current.onChange(MOVE, 1));
    act(() => result.current.onChangeCommitted(RELEASE, 1));

    expect(onChange.mock.calls).toEqual([[1]]);
  });

  it("tells nothing for a press that never moved, whatever value the slider commits", () => {
    const onChange = jest.fn();
    const { result } = renderHook(() => useThrottledDraft(2, 1, onChange, 200));

    act(() => result.current.onChangeCommitted(RELEASE, 2));
    act(() => result.current.onChangeCommitted(RELEASE, 7));

    expect(onChange).not.toHaveBeenCalled();
    expect(result.current.value).toBe(2);
  });

  it("never tells a stale value after the owner resets", () => {
    const onChange = jest.fn();
    const { result, rerender } = renderHook(({ value }) => useThrottledDraft(value, 1, onChange, 200), {
      initialProps: { value: 0 },
    });

    act(() => result.current.onChange(MOVE, 4));
    act(() => result.current.onChangeCommitted(RELEASE, 4));
    rerender({ value: 4 });
    rerender({ value: 0 });
    // MUI reports the last value it changed to, which is no longer what the owner holds.
    act(() => result.current.onChangeCommitted(RELEASE, 4));

    expect(onChange.mock.calls).toEqual([[4]]);
    expect(result.current.value).toBe(0);
  });

  it("tells a value again once the owner moved away from it", () => {
    const onChange = jest.fn();
    const { result, rerender } = renderHook(({ value }) => useThrottledDraft(value, 1, onChange, 200), {
      initialProps: { value: 0 },
    });

    act(() => result.current.onChange(MOVE, 4));
    act(() => result.current.onChangeCommitted(RELEASE, 4));
    rerender({ value: 4 });
    rerender({ value: 0 });
    act(() => jest.advanceTimersByTime(200));
    act(() => result.current.onChange(MOVE, 4));
    act(() => result.current.onChangeCommitted(RELEASE, 4));

    expect(onChange.mock.calls).toEqual([[4], [4]]);
  });

  it("drops a draft the owner overrides mid-drag, and never sends what was pending", () => {
    const onChange = jest.fn();
    const { result, rerender } = renderHook(({ value }) => useThrottledDraft(value, 1, onChange, 200), {
      initialProps: { value: 0 },
    });

    act(() => result.current.onChange(MOVE, 1));
    rerender({ value: 1 });
    // A gesture the browser cancels never commits.
    act(() => result.current.onChange(MOVE, 2));
    rerender({ value: 9 });
    act(() => jest.advanceTimersByTime(1000));

    expect(result.current.value).toBe(9);
    expect(onChange.mock.calls).toEqual([[1]]);
  });

  // The grass density slider shows a scale the settings store as a spacing, and 0.85 comes back 0.8500000000000001.
  it("keeps the gesture when the owner hands back an inexact round trip of what it was told", () => {
    function roundTrip(scale: number): number {
      return 0.6 / (0.6 / scale);
    }

    const onChange = jest.fn();
    const { result, rerender } = renderHook(({ value }) => useThrottledDraft(value, 0.05, onChange, 200), {
      initialProps: { value: 1 },
    });

    expect(roundTrip(0.85)).not.toBe(0.85);

    act(() => result.current.onChange(MOVE, 0.85));
    act(() => result.current.onChange(MOVE, 0.9));
    rerender({ value: roundTrip(0.85) });
    act(() => result.current.onChangeCommitted(RELEASE, 0.9));

    expect(onChange.mock.calls).toEqual([[0.85], [0.9]]);
  });

  it("keeps the draft while the owner follows it", () => {
    const onChange = jest.fn();
    const { result, rerender } = renderHook(({ value }) => useThrottledDraft(value, 1, onChange, 200), {
      initialProps: { value: 0 },
    });

    act(() => result.current.onChange(MOVE, 1));
    rerender({ value: 1 });
    act(() => result.current.onChange(MOVE, 2));

    expect(result.current.value).toBe(2);
  });

  it("throttles held keys like a drag, and still tells the last one", () => {
    const onChange = jest.fn();
    const { result, rerender } = renderHook(({ value }) => useThrottledDraft(value, 1, onChange, 200), {
      initialProps: { value: 0 },
    });

    for (const step of [1, 2, 3, 4]) {
      act(() => result.current.onChange(KEY, step));
      act(() => result.current.onChangeCommitted(KEY, step));
      act(() => jest.advanceTimersByTime(30));
    }

    expect(onChange.mock.calls).toEqual([[1]]);
    expect(result.current.value).toBe(4);

    rerender({ value: 1 });
    act(() => jest.advanceTimersByTime(200));

    expect(onChange.mock.calls).toEqual([[1], [4]]);

    rerender({ value: 4 });

    expect(result.current.value).toBe(4);
  });

  it("tells what it was dragged to when taken down mid-drag", () => {
    const onChange = jest.fn();
    const { result, unmount } = renderHook(() => useThrottledDraft(0, 1, onChange, 200));

    act(() => result.current.onChange(MOVE, 1));
    act(() => result.current.onChange(MOVE, 2));
    unmount();

    expect(onChange.mock.calls).toEqual([[1], [2]]);
  });
});
