import { afterEach, beforeEach, describe, expect, it, jest } from "@jest/globals";
import { act, fireEvent } from "@testing-library/react";
import { userEvent } from "@testing-library/user-event";

import { renderWithTheme } from "#/testing/render";

import { AudioPlayer } from "./AudioPlayer";

const SRC: string = "blob:mock/sound";

/**
 * jsdom implements neither playback nor a canvas, which is also the environment the component has to
 * survive in a webview that refuses to decode a sound. Playback is stubbed so the transport can be
 * driven; the canvas is left unimplemented by the environment on purpose, so these tests cover the
 * degraded path.
 */
beforeEach(() => {
  // `paused` is tracked because the component asks the element what it is doing rather than keeping its
  // own idea of it, and jsdom's stays true forever once `play` is stubbed out.
  let isPaused: boolean = true;

  jest.spyOn(window.HTMLMediaElement.prototype, "paused", "get").mockImplementation(() => isPaused);

  jest.spyOn(window.HTMLMediaElement.prototype, "play").mockImplementation(function (this: HTMLMediaElement) {
    isPaused = false;
    this.dispatchEvent(new Event("play"));

    return Promise.resolve();
  });

  jest.spyOn(window.HTMLMediaElement.prototype, "pause").mockImplementation(function (this: HTMLMediaElement) {
    isPaused = true;
    this.dispatchEvent(new Event("pause"));
  });
});

afterEach(() => {
  jest.restoreAllMocks();
});

function ignoreVolume(): void {}

function getAudio(container: HTMLElement): HTMLAudioElement {
  return container.querySelector("audio") as HTMLAudioElement;
}

describe("AudioPlayer", () => {
  it("explains a rejected playback request and clears the error on a successful retry", async () => {
    jest
      .spyOn(window.HTMLMediaElement.prototype, "play")
      .mockRejectedValueOnce(new DOMException("Unsupported source", "NotSupportedError"));

    const { getByLabelText, findByRole, queryByRole } = renderWithTheme(
      <AudioPlayer volume={1} onVolumeChange={ignoreVolume} src={SRC} />
    );

    await userEvent.click(getByLabelText("Play"));

    expect(await findByRole("alert")).toHaveTextContent("This audio format is not supported or the file is damaged.");
    expect(getByLabelText("Play")).toBeInTheDocument();

    await userEvent.click(getByLabelText("Play"));

    expect(queryByRole("alert")).not.toBeInTheDocument();
    expect(getByLabelText("Pause")).toBeInTheDocument();
  });

  it.each([
    [2, "Could not read this audio"],
    [3, "Could not decode this audio"],
    [4, "This audio format is not supported or the file is unavailable"],
  ])("explains media error %s and reloads the source when retried", async (code, message) => {
    const { container, getByLabelText, getByRole, queryByRole } = renderWithTheme(
      <AudioPlayer volume={1} onVolumeChange={ignoreVolume} src={SRC} />
    );
    const audio: HTMLAudioElement = getAudio(container);
    const load = jest.spyOn(audio, "load").mockImplementation(() => undefined);

    Object.defineProperty(audio, "error", { configurable: true, value: { code, message: "Browser detail" } });

    fireEvent.error(audio);

    expect(getByRole("alert")).toHaveTextContent(String(message));
    expect(getByLabelText("Play")).toBeInTheDocument();

    await userEvent.click(getByLabelText("Play"));

    expect(load).toHaveBeenCalledTimes(1);
    expect(queryByRole("alert")).not.toBeInTheDocument();
  });

  it.each([false, true])(
    "ignores old playback failures after changing sounds (media error: %s)",
    async (hasMediaError) => {
      let rejectPlay: (error: Error) => void;

      jest.spyOn(window.HTMLMediaElement.prototype, "play").mockImplementationOnce(
        () =>
          new Promise<void>((_, reject) => {
            rejectPlay = reject;
          })
      );

      const { container, getByLabelText, getByRole, queryByRole, rerender } = renderWithTheme(
        <AudioPlayer volume={1} onVolumeChange={ignoreVolume} src={SRC} />
      );

      await userEvent.click(getByLabelText("Play"));
      if (hasMediaError) {
        fireEvent.error(getAudio(container));

        expect(getByRole("alert")).toBeInTheDocument();
      }

      rerender(
        <>
          <AudioPlayer volume={1} onVolumeChange={ignoreVolume} src={"blob:mock/other"} />
        </>
      );

      await act(async () => rejectPlay(new Error("Old source failed")));

      expect(queryByRole("alert")).not.toBeInTheDocument();
      expect(getByLabelText("Play")).toBeInTheDocument();
    }
  );

  it("still renders a usable transport where nothing can be decoded or drawn", () => {
    // The point of drawing our own player is that it degrades: no waveform, but the sound still plays.
    const { getByLabelText } = renderWithTheme(
      <AudioPlayer volume={1} onVolumeChange={ignoreVolume} src={SRC} bytes={null} />
    );

    expect(getByLabelText("Play")).toBeInTheDocument();
    expect(getByLabelText("Seek")).toBeInTheDocument();
  });

  it("follows the element rather than assuming its own button worked", async () => {
    const { container, getByLabelText, queryByLabelText } = renderWithTheme(
      <AudioPlayer volume={1} onVolumeChange={ignoreVolume} src={SRC} />
    );

    await userEvent.click(getByLabelText("Play"));

    expect(window.HTMLMediaElement.prototype.play).toHaveBeenCalled();
    expect(getByLabelText("Pause")).toBeInTheDocument();

    // A sound that stops on its own must leave the button offering to play it again.
    act(() => void fireEvent.ended(getAudio(container)));

    expect(queryByLabelText("Pause")).not.toBeInTheDocument();
    expect(getByLabelText("Play")).toBeInTheDocument();
  });

  it("pauses a sound that is already playing", async () => {
    const { getByLabelText } = renderWithTheme(<AudioPlayer volume={1} onVolumeChange={ignoreVolume} src={SRC} />);

    await userEvent.click(getByLabelText("Play"));
    await userEvent.click(getByLabelText("Pause"));

    expect(window.HTMLMediaElement.prototype.pause).toHaveBeenCalled();
    expect(getByLabelText("Play")).toBeInTheDocument();
  });

  it("reports position against duration once the element knows both", () => {
    const { container, getByText } = renderWithTheme(
      <AudioPlayer volume={1} onVolumeChange={ignoreVolume} src={SRC} />
    );
    const audio: HTMLAudioElement = getAudio(container);

    // Before metadata arrives the element reports NaN, which must not reach the readout.
    expect(getByText("00:00 / 00:00")).toBeInTheDocument();

    jest.spyOn(audio, "duration", "get").mockReturnValue(65);
    act(() => void fireEvent.loadedMetadata(audio));

    jest.spyOn(audio, "currentTime", "get").mockReturnValue(9);
    act(() => void fireEvent.timeUpdate(audio));

    expect(getByText("00:09 / 01:05")).toBeInTheDocument();
  });

  it("seeks proportionally to where the waveform was clicked", () => {
    const { container, getByLabelText } = renderWithTheme(
      <AudioPlayer volume={1} onVolumeChange={ignoreVolume} src={SRC} />
    );
    const audio: HTMLAudioElement = getAudio(container);
    const waveform: HTMLElement = getByLabelText("Seek");

    jest.spyOn(audio, "duration", "get").mockReturnValue(100);
    act(() => void fireEvent.loadedMetadata(audio));

    // jsdom lays nothing out, so the strip has to be given a width to click within.
    jest.spyOn(waveform, "getBoundingClientRect").mockReturnValue({ left: 20, width: 200 } as DOMRect);

    fireEvent.click(waveform, { clientX: 70 });

    expect(audio.currentTime).toBe(25);
  });

  it("ignores a seek on a sound whose length is still unknown", () => {
    const { container, getByLabelText } = renderWithTheme(
      <AudioPlayer volume={1} onVolumeChange={ignoreVolume} src={SRC} />
    );
    const audio: HTMLAudioElement = getAudio(container);
    const waveform: HTMLElement = getByLabelText("Seek");
    const setCurrentTime = jest.fn();

    // Asserting on the setter rather than the value: seeking a zero length sound computes zero, so the
    // value alone cannot tell a skipped seek from one that ran and happened to land on the start.
    jest.spyOn(audio, "currentTime", "set").mockImplementation(setCurrentTime);
    jest.spyOn(waveform, "getBoundingClientRect").mockReturnValue({ left: 0, width: 200 } as DOMRect);

    fireEvent.click(waveform, { clientX: 100 });

    expect(setCurrentTime).not.toHaveBeenCalled();
  });

  it("starts over when another sound is selected", () => {
    const { container, getByLabelText, getByText, rerender } = renderWithTheme(
      <AudioPlayer volume={1} onVolumeChange={ignoreVolume} src={SRC} />
    );
    const audio: HTMLAudioElement = getAudio(container);

    jest.spyOn(audio, "duration", "get").mockReturnValue(65);
    act(() => void fireEvent.loadedMetadata(audio));
    act(() => void fireEvent.play(audio));

    rerender(
      <>
        <AudioPlayer volume={1} onVolumeChange={ignoreVolume} src={"blob:mock/other"} />
      </>
    );

    // Replacing the source stops playback without an event, so nothing about the last sound may linger.
    expect(getByText("00:00 / 00:00")).toBeInTheDocument();
    expect(getByLabelText("Play")).toBeInTheDocument();
  });

  it("loops on demand, because most archived sounds are ambience", async () => {
    const { container, getByLabelText } = renderWithTheme(
      <AudioPlayer volume={1} onVolumeChange={ignoreVolume} src={SRC} />
    );

    expect(getAudio(container).loop).toBe(false);

    await userEvent.click(getByLabelText("Loop"));

    expect(getAudio(container).loop).toBe(true);
  });

  it("seeks repeatedly from the keyboard and clamps to the sound's bounds", async () => {
    const { container, getByRole } = renderWithTheme(
      <AudioPlayer volume={1} onVolumeChange={ignoreVolume} src={SRC} />
    );
    const audio = getAudio(container);
    const waveform = getByRole("slider", { name: "Seek" });

    jest.spyOn(audio, "duration", "get").mockReturnValue(12);
    fireEvent.loadedMetadata(audio);

    await userEvent.tab();

    expect(waveform).toHaveFocus();

    await userEvent.keyboard("{ArrowRight}{ArrowRight}{ArrowRight}");

    expect(audio.currentTime).toBe(12);
    expect(waveform).toHaveAttribute("aria-valuenow", "12");
    expect(waveform).toHaveAttribute("aria-valuetext", "00:12 of 00:12");

    await userEvent.keyboard("{ArrowLeft}{ArrowDown}");

    expect(audio.currentTime).toBe(2);

    await userEvent.keyboard("{Home}{ArrowLeft}");

    expect(audio.currentTime).toBe(0);

    await userEvent.keyboard("{End}");

    expect(audio.currentTime).toBe(12);
  });

  it("keeps playback shortcuts available when duration is unknown but does not seek", async () => {
    const { container, getByRole } = renderWithTheme(
      <AudioPlayer volume={1} onVolumeChange={ignoreVolume} src={SRC} />
    );
    const audio = getAudio(container);
    const setCurrentTime = jest.spyOn(audio, "currentTime", "set");

    jest.spyOn(audio, "duration", "get").mockReturnValue(Infinity);
    fireEvent.loadedMetadata(audio);

    expect(getByRole("slider", { name: "Seek" })).toHaveAttribute("aria-valuemax", "0");

    await userEvent.tab();
    await userEvent.keyboard("{ArrowRight}{Home}{End} ");

    expect(setCurrentTime).not.toHaveBeenCalled();
    expect(getByRole("button", { name: "Pause" })).toBeInTheDocument();

    await userEvent.keyboard("{Enter}");

    expect(getByRole("button", { name: "Play" })).toBeInTheDocument();
  });

  it("ignores a click while the waveform has no width", () => {
    const { container, getByRole } = renderWithTheme(
      <AudioPlayer volume={1} onVolumeChange={ignoreVolume} src={SRC} />
    );
    const audio = getAudio(container);
    const setCurrentTime = jest.spyOn(audio, "currentTime", "set");

    jest.spyOn(audio, "duration", "get").mockReturnValue(100);
    fireEvent.loadedMetadata(audio);
    fireEvent.click(getByRole("slider", { name: "Seek" }), { clientX: 100 });

    expect(setCurrentTime).not.toHaveBeenCalled();
  });

  it("tells the caller where the volume slider is set", () => {
    const onVolumeChange = jest.fn<(volume: number) => void>();
    const { getByLabelText } = renderWithTheme(<AudioPlayer volume={1} onVolumeChange={onVolumeChange} src={SRC} />);

    fireEvent.change(getByLabelText("Volume"), { target: { value: "0.25" } });

    expect(onVolumeChange).toHaveBeenCalledWith(0.25);
  });

  it("gives a new sound's element the level it is handed, not just the slider", () => {
    // Selecting another file unmounts this player and builds a new one, so the level comes from the caller.
    const { container, getByLabelText } = renderWithTheme(
      <AudioPlayer volume={0.5} onVolumeChange={ignoreVolume} src={"blob:mock/other"} />
    );

    expect(getAudio(container).volume).toBeCloseTo(0.5);
    expect(getByLabelText("Volume")).toHaveValue("0.5");
  });
});
