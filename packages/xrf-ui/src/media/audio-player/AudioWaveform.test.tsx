import { afterEach, beforeEach, describe, expect, it, jest } from "@jest/globals";
import { act } from "@testing-library/react";
import { userEvent } from "@testing-library/user-event";

import { renderWithTheme } from "#/testing/render";

import { AudioWaveform } from "./AudioWaveform";

/** Stands in for the callbacks and observers a case does not watch. */
function noop(): void {}

const ORIGINAL_RESIZE_OBSERVER = global.ResizeObserver;

/** A sound's peak table, each stretch at `value`. */
function peaks(value: number): Float32Array {
  return new Float32Array([value, value, value, value]);
}

let width: number = 100;
let resize: () => void = noop;

class TestResizeObserver {
  public constructor(callback: () => void) {
    resize = callback;
  }

  public observe(): void {}
  public unobserve(): void {}
  public disconnect(): void {
    resize = noop;
  }
}

const fillRect = jest.fn();
const clearRect = jest.fn();

beforeEach(() => {
  width = 100;

  fillRect.mockClear();
  clearRect.mockClear();

  global.ResizeObserver = TestResizeObserver as unknown as typeof ResizeObserver;

  jest.spyOn(HTMLCanvasElement.prototype, "clientWidth", "get").mockImplementation(() => width);
  jest.spyOn(HTMLCanvasElement.prototype, "clientHeight", "get").mockReturnValue(96);
  jest
    .spyOn(HTMLCanvasElement.prototype, "getContext")
    .mockReturnValue({ fillRect, clearRect } as unknown as CanvasRenderingContext2D);
});

afterEach(() => {
  jest.restoreAllMocks();
  global.ResizeObserver = ORIGINAL_RESIZE_OBSERVER;
});

describe("AudioWaveform", () => {
  it("redraws playback progress without resetting unchanged canvas dimensions", () => {
    const table: Float32Array = peaks(0.5);
    const { getByRole, rerender } = renderWithTheme(
      <AudioWaveform peaks={table} position={0} duration={10} onSeek={noop} onTogglePlay={noop} />
    );

    const canvas = getByRole("slider", { name: "Seek" });
    const observer: MutationObserver = new MutationObserver(noop);

    observer.observe(canvas, { attributes: true, attributeFilter: ["width", "height"] });
    clearRect.mockClear();
    fillRect.mockClear();

    rerender(
      <>
        <AudioWaveform peaks={table} position={5} duration={10} onSeek={noop} onTogglePlay={noop} />
      </>
    );

    const mutations: Array<MutationRecord> = observer.takeRecords();

    observer.disconnect();

    expect(getByRole("slider", { name: "Seek" })).toBe(canvas);
    expect(canvas).toHaveAttribute("aria-valuenow", "5");
    expect(clearRect).toHaveBeenCalledTimes(1);
    expect(fillRect).toHaveBeenCalledTimes(50);
    expect(mutations).toHaveLength(0);
  });

  it("redraws a waveform at its new resolution from the same peaks", () => {
    const { getByRole } = renderWithTheme(
      <AudioWaveform peaks={peaks(0.5)} position={0} duration={10} onSeek={noop} onTogglePlay={noop} />
    );

    const canvas = getByRole("slider", { name: "Seek" });

    expect(canvas).toHaveAttribute("width", "100");
    expect(canvas).toHaveAttribute("height", "96");
    expect(fillRect).toHaveBeenLastCalledWith(98, 24, 1, 48);

    fillRect.mockClear();

    act(() => {
      width = 200;
      resize();
    });

    expect(canvas).toHaveAttribute("width", "200");
    expect(clearRect).toHaveBeenLastCalledWith(0, 0, 200, 96);
    expect(fillRect).toHaveBeenCalledTimes(100);
    expect(fillRect).toHaveBeenLastCalledWith(198, 24, 1, 48);
  });

  it("draws a flat line while the peaks are unknown", () => {
    const { rerender } = renderWithTheme(
      <AudioWaveform peaks={peaks(1)} position={0} duration={10} onSeek={noop} onTogglePlay={noop} />
    );

    rerender(
      <>
        <AudioWaveform peaks={null} position={0} duration={10} onSeek={noop} onTogglePlay={noop} />
      </>
    );

    expect(fillRect).toHaveBeenLastCalledWith(0, 48, 100, 1);
  });

  it("leaves the seek surface and playback shortcut usable without peaks", async () => {
    const onTogglePlay = jest.fn();
    const { getByRole } = renderWithTheme(
      <AudioWaveform peaks={null} position={0} duration={10} onSeek={noop} onTogglePlay={onTogglePlay} />
    );

    await userEvent.tab();

    expect(getByRole("slider", { name: "Seek" })).toHaveFocus();

    await userEvent.keyboard(" ");

    expect(onTogglePlay).toHaveBeenCalledTimes(1);
  });
});
