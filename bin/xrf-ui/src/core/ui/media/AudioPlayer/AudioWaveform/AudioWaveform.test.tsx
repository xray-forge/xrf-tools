import { afterEach, beforeEach, describe, expect, it, jest } from "@jest/globals";
import { act } from "@testing-library/react";
import { userEvent } from "@testing-library/user-event";

import { renderWithProviders } from "@/fixtures/utils/render";
import { noop } from "@/lib/callbacks/noop";

import { AudioWaveform } from "./AudioWaveform";

const BYTES: Uint8Array = new Uint8Array([1, 2]);
const ORIGINAL_AUDIO_CONTEXT = Object.getOwnPropertyDescriptor(global, "AudioContext");
const ORIGINAL_RESIZE_OBSERVER = global.ResizeObserver;

/** Decoding resolves under the test's control so a source can change while it is pending. */
class TestAudioContext {
  public resolve: (buffer: AudioBuffer) => void = noop;
  public reject: (error: Error) => void = noop;
  public close = jest.fn<() => Promise<void>>().mockResolvedValue(undefined);
  public decodeAudioData = jest.fn(
    () =>
      new Promise<AudioBuffer>((resolve, reject) => {
        this.resolve = resolve;
        this.reject = reject;
      })
  );

  public constructor() {
    contexts.push(this);
  }
}

let contexts: Array<TestAudioContext> = [];
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

function samples(value: number): AudioBuffer {
  return { getChannelData: () => new Float32Array([value, value, value, value]) } as unknown as AudioBuffer;
}

beforeEach(() => {
  contexts = [];
  width = 100;
  fillRect.mockClear();
  clearRect.mockClear();
  Object.defineProperty(global, "AudioContext", { configurable: true, writable: true, value: TestAudioContext });
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

  if (ORIGINAL_AUDIO_CONTEXT) {
    Object.defineProperty(global, "AudioContext", ORIGINAL_AUDIO_CONTEXT);
  } else {
    Reflect.deleteProperty(global, "AudioContext");
  }
});

describe("AudioWaveform", () => {
  it("redraws a paused waveform at its new resolution without decoding again", async () => {
    const { getByRole } = renderWithProviders(
      <AudioWaveform src={"first"} bytes={BYTES} position={0} duration={10} onSeek={noop} onTogglePlay={noop} />
    );

    await act(async () => contexts[0].resolve(samples(0.5)));

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
    expect(contexts).toHaveLength(1);
    expect(contexts[0].decodeAudioData).toHaveBeenCalledTimes(1);
    expect(contexts[0].close).toHaveBeenCalledTimes(1);
  });

  it("ignores a previous source that decodes after its replacement", async () => {
    const { rerender } = renderWithProviders(
      <AudioWaveform src={"first"} bytes={BYTES} position={0} duration={10} onSeek={noop} onTogglePlay={noop} />
    );

    // Source identity matters even when its caller reuses the byte array.
    rerender(
      <>
        <AudioWaveform src={"second"} bytes={BYTES} position={0} duration={10} onSeek={noop} onTogglePlay={noop} />
      </>
    );

    await act(async () => contexts[1].resolve(samples(0.5)));

    expect(fillRect).toHaveBeenLastCalledWith(98, 24, 1, 48);
    fillRect.mockClear();

    await act(async () => contexts[0].resolve(samples(1)));

    expect(fillRect).not.toHaveBeenCalled();
    expect(contexts[0].close).toHaveBeenCalledTimes(1);
    expect(contexts[1].close).toHaveBeenCalledTimes(1);
  });

  it("clears the old picture while a new source is decoding", async () => {
    const { rerender } = renderWithProviders(
      <AudioWaveform src={"first"} bytes={BYTES} position={0} duration={10} onSeek={noop} onTogglePlay={noop} />
    );

    await act(async () => contexts[0].resolve(samples(1)));

    rerender(
      <>
        <AudioWaveform
          src={"second"}
          bytes={new Uint8Array([3])}
          position={0}
          duration={10}
          onSeek={noop}
          onTogglePlay={noop}
        />
      </>
    );

    expect(fillRect).toHaveBeenLastCalledWith(0, 48, 100, 1);

    await act(async () => contexts[1].resolve(samples(0.5)));
  });

  it("leaves the seek surface and playback shortcut usable when decoding fails", async () => {
    const onTogglePlay = jest.fn();
    const { getByRole } = renderWithProviders(
      <AudioWaveform src={"first"} bytes={BYTES} position={0} duration={10} onSeek={noop} onTogglePlay={onTogglePlay} />
    );

    await act(async () => contexts[0].reject(new Error("Unsupported audio")));
    await userEvent.tab();

    expect(getByRole("slider", { name: "Seek" })).toHaveFocus();
    expect(fillRect).toHaveBeenLastCalledWith(0, 48, 100, 1);

    await userEvent.keyboard(" ");

    expect(onTogglePlay).toHaveBeenCalledTimes(1);
    expect(contexts[0].close).toHaveBeenCalledTimes(1);
  });

  it("releases a decode that finishes after unmount without drawing it", async () => {
    const { unmount } = renderWithProviders(
      <AudioWaveform src={"first"} bytes={BYTES} position={0} duration={10} onSeek={noop} onTogglePlay={noop} />
    );

    unmount();
    fillRect.mockClear();

    await act(async () => contexts[0].resolve(samples(1)));

    expect(fillRect).not.toHaveBeenCalled();
    expect(contexts[0].close).toHaveBeenCalledTimes(1);
  });
});
