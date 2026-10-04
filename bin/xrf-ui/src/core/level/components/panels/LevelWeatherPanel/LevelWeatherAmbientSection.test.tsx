import { describe, expect, it, jest } from "@jest/globals";
import { userEvent } from "@testing-library/user-event";

import { RenderAmbientReport } from "@/core/ipc/types/xrf-renderer";
import { renderWithProviders } from "@/fixtures/utils/render";

import { LevelWeatherAmbientSection } from "./LevelWeatherAmbientSection";

const PLAYING: RenderAmbientReport = {
  effect: { name: "effect_3", particles: "nature\\fog_tornado_00", remaining: 6.2 },
  isIndoors: false,
  wait: 21.5,
};

describe("LevelWeatherAmbientSection", () => {
  it("says what plays, how long it has left and when the next may start, and plays one at once", async () => {
    const onPlay = jest.fn<() => void>();
    const { getByText, getByRole } = renderWithProviders(
      <LevelWeatherAmbientSection ambient={PLAYING} isPlayed={true} onPlay={onPlay} />
    );

    expect(getByText("effect_3, nature\\fog_tornado_00")).toBeInTheDocument();
    expect(getByText("7 s")).toBeInTheDocument();
    expect(getByText("In 22 s")).toBeInTheDocument();
    expect(getByText("Outdoors")).toBeInTheDocument();

    await userEvent.click(getByRole("button", { name: "Play one now" }));

    expect(onPlay).toHaveBeenCalledTimes(1);
  });

  it("says an effect is dying out once its life is up and the next waits for it", () => {
    const { getByText } = renderWithProviders(
      <LevelWeatherAmbientSection
        ambient={{ ...PLAYING, effect: { ...PLAYING.effect!, remaining: 0 }, wait: 0 }}
        isPlayed={true}
        onPlay={() => {}}
      />
    );

    expect(getByText("Dying out")).toBeInTheDocument();
    expect(getByText("Once this one is gone")).toBeInTheDocument();
  });

  it("plays none indoors", () => {
    const { getByText, getByRole } = renderWithProviders(
      <LevelWeatherAmbientSection
        ambient={{ effect: null, isIndoors: true, wait: 0 }}
        isPlayed={true}
        onPlay={() => {}}
      />
    );

    expect(getByText("Nothing")).toBeInTheDocument();
    expect(getByText("Indoors, none start")).toBeInTheDocument();
    expect(getByRole("button", { name: "Play one now" })).toBeDisabled();
  });

  it("says when they are switched off or not read yet", () => {
    const { getByText, rerender } = renderWithProviders(
      <LevelWeatherAmbientSection ambient={PLAYING} isPlayed={false} onPlay={() => {}} />
    );

    expect(getByText("Switched off in the Particles menu.")).toBeInTheDocument();

    rerender(<LevelWeatherAmbientSection ambient={null} isPlayed={true} onPlay={() => {}} />);

    expect(getByText("Waiting for the level's particles.")).toBeInTheDocument();
  });
});
