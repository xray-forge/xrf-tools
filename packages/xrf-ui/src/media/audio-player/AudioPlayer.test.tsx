import { describe, expect, it, jest } from "@jest/globals";
import { fireEvent } from "@testing-library/react";
import { userEvent } from "@testing-library/user-event";

import { renderWithTheme } from "#/testing/render";

import { IAudioTransport, IAudioTransportState } from "./audio-transport";
import { AudioPlayer } from "./AudioPlayer";

const STOPPED: IAudioTransportState = {
  playback: "stopped",
  position: 0,
  duration: 65,
  isLooped: false,
  failure: null,
};

/** A transport that records what the player asks and plays nothing: the state is whatever a case hands it. */
function mockTransport(state: Partial<IAudioTransportState> = {}): IAudioTransport {
  return {
    state: { ...STOPPED, ...state },
    peaks: null,
    play: jest.fn(),
    pause: jest.fn(),
    seek: jest.fn(),
    setLooped: jest.fn(),
  };
}

function ignoreVolume(): void {}

describe("AudioPlayer", () => {
  it("asks a stopped, paused or ended transport to play", async () => {
    for (const playback of ["stopped", "paused", "ended"] as const) {
      const transport: IAudioTransport = mockTransport({ playback });
      const { getByLabelText, unmount } = renderWithTheme(
        <AudioPlayer transport={transport} volume={1} onVolumeChange={ignoreVolume} />
      );

      await userEvent.click(getByLabelText("Play"));

      expect(transport.play).toHaveBeenCalledTimes(1);
      expect(transport.pause).not.toHaveBeenCalled();

      unmount();
    }
  });

  it("asks a playing transport to pause", async () => {
    const transport: IAudioTransport = mockTransport({ playback: "playing" });
    const { getByLabelText } = renderWithTheme(
      <AudioPlayer transport={transport} volume={1} onVolumeChange={ignoreVolume} />
    );

    await userEvent.click(getByLabelText("Pause"));

    expect(transport.pause).toHaveBeenCalledTimes(1);
  });

  it("follows the transport rather than its own button", () => {
    const transport: IAudioTransport = mockTransport({ playback: "playing" });
    const { getByLabelText, rerender } = renderWithTheme(
      <AudioPlayer transport={transport} volume={1} onVolumeChange={ignoreVolume} />
    );

    expect(getByLabelText("Pause")).toBeInTheDocument();

    // A sound that ends on its own must leave the button offering to play it again.
    rerender(
      <>
        <AudioPlayer
          transport={{ ...transport, state: { ...transport.state, playback: "ended" } }}
          volume={1}
          onVolumeChange={ignoreVolume}
        />
      </>
    );

    expect(getByLabelText("Play")).toBeInTheDocument();
  });

  it("reports position against duration", () => {
    const { getByText } = renderWithTheme(
      <AudioPlayer transport={mockTransport({ position: 9 })} volume={1} onVolumeChange={ignoreVolume} />
    );

    expect(getByText("00:09 / 01:05")).toBeInTheDocument();
  });

  it("shows why the transport failed", () => {
    const { getByRole } = renderWithTheme(
      <AudioPlayer
        transport={mockTransport({ failure: "Could not play: every voice is playing" })}
        volume={1}
        onVolumeChange={ignoreVolume}
      />
    );

    expect(getByRole("alert")).toHaveTextContent("Could not play: every voice is playing");
  });

  it("seeks proportionally to where the waveform was clicked", () => {
    const transport: IAudioTransport = mockTransport({ duration: 100 });
    const { getByLabelText } = renderWithTheme(
      <AudioPlayer transport={transport} volume={1} onVolumeChange={ignoreVolume} />
    );
    const waveform: HTMLElement = getByLabelText("Seek");

    // jsdom lays nothing out, so the strip has to be given a width to click within.
    jest.spyOn(waveform, "getBoundingClientRect").mockReturnValue({ left: 20, width: 200 } as DOMRect);

    fireEvent.click(waveform, { clientX: 70 });

    expect(transport.seek).toHaveBeenCalledWith(25);
  });

  it("seeks from the keyboard, clamped to the sound's bounds", async () => {
    const transport: IAudioTransport = mockTransport({ duration: 12, position: 10 });
    const { getByRole } = renderWithTheme(
      <AudioPlayer transport={transport} volume={1} onVolumeChange={ignoreVolume} />
    );

    await userEvent.tab();

    expect(getByRole("slider", { name: "Seek" })).toHaveFocus();

    await userEvent.keyboard("{ArrowRight}{ArrowLeft}{Home}{End}");

    expect(transport.seek).toHaveBeenNthCalledWith(1, 12);
    expect(transport.seek).toHaveBeenNthCalledWith(2, 5);
    expect(transport.seek).toHaveBeenNthCalledWith(3, 0);
    expect(transport.seek).toHaveBeenNthCalledWith(4, 12);
  });

  it("never seeks a sound whose length is unknown", async () => {
    const transport: IAudioTransport = mockTransport({ duration: 0 });
    const { getByRole } = renderWithTheme(
      <AudioPlayer transport={transport} volume={1} onVolumeChange={ignoreVolume} />
    );

    expect(getByRole("slider", { name: "Seek" })).toHaveAttribute("aria-valuemax", "0");

    await userEvent.tab();
    await userEvent.keyboard("{ArrowRight}{Home}{End}");

    expect(transport.seek).not.toHaveBeenCalled();
  });

  it("toggles looping on the transport, because most archived sounds are ambience", async () => {
    const transport: IAudioTransport = mockTransport({ isLooped: false });
    const { getByLabelText } = renderWithTheme(
      <AudioPlayer transport={transport} volume={1} onVolumeChange={ignoreVolume} />
    );

    expect(getByLabelText("Loop")).toHaveAttribute("aria-pressed", "false");

    await userEvent.click(getByLabelText("Loop"));

    expect(transport.setLooped).toHaveBeenCalledWith(true);
  });

  it("tells the caller where the volume slider is set, and shows the level it is handed", () => {
    const onVolumeChange = jest.fn<(volume: number) => void>();
    const { getByLabelText } = renderWithTheme(
      <AudioPlayer transport={mockTransport()} volume={0.5} onVolumeChange={onVolumeChange} />
    );

    expect(getByLabelText("Volume")).toHaveValue("0.5");

    fireEvent.change(getByLabelText("Volume"), { target: { value: "0.25" } });

    expect(onVolumeChange).toHaveBeenCalledWith(0.25);
  });
});
