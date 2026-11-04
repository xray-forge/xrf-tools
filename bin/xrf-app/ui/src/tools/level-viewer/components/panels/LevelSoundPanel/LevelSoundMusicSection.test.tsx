import { describe, expect, it, jest } from "@jest/globals";
import { userEvent } from "@testing-library/user-event";
import { WorldMusicCommand, WorldSoundMusicReport } from "@xrf/ipc/types/xrf-world";

import { renderWithProviders } from "@/fixtures/utils/render";
import { LevelSoundMusicSection } from "@/tools/level-viewer/components/panels/LevelSoundPanel/LevelSoundMusicSection";

/** The world's answer to any command: taken. */
function answer(_: WorldMusicCommand): Promise<boolean> {
  return Promise.resolve(true);
}

function music(report: Partial<WorldSoundMusicReport> = {}): WorldSoundMusicReport {
  return {
    track: "music_01",
    position: 65_000,
    length: 180_000,
    wait: 0,
    chosen: null,
    tracks: ["music_01", "music_02"],
    ...report,
  };
}

describe("LevelSoundMusicSection", () => {
  it("plays a track chosen by hand, and the schedule back", async () => {
    const onCommand = jest.fn(answer);
    const { getByRole, findByRole, rerender } = renderWithProviders(
      <LevelSoundMusicSection music={music()} onCommand={onCommand} />
    );

    expect(getByRole("combobox", { name: "Track" })).toHaveTextContent("By schedule");

    await userEvent.click(getByRole("combobox", { name: "Track" }));
    await userEvent.click(await findByRole("option", { name: "music_02" }));

    expect(onCommand).toHaveBeenLastCalledWith({ kind: "play", track: "music_02" });

    rerender(<LevelSoundMusicSection music={music({ chosen: "music_02", track: "music_02" })} onCommand={onCommand} />);

    expect(getByRole("combobox", { name: "Track" })).toHaveTextContent("music_02");

    await userEvent.click(getByRole("combobox", { name: "Track" }));
    await userEvent.click(await findByRole("option", { name: "By schedule" }));

    expect(onCommand).toHaveBeenLastCalledWith({ kind: "schedule" });
  });

  it("shows how far the track has played and plays it from where the slider is let go", async () => {
    const onCommand = jest.fn(answer);
    const { getByRole, getByText } = renderWithProviders(
      <LevelSoundMusicSection music={music()} onCommand={onCommand} />
    );

    expect(getByText("1:05 / 3:00")).toBeInTheDocument();

    getByRole("slider", { name: "Play the track from" }).focus();
    await userEvent.keyboard("{ArrowRight}");

    expect(onCommand).toHaveBeenLastCalledWith({ kind: "seek", position: 66_000 });
  });

  // A report made before the world took the seek still stands where the track was: the slider keeps the point let go of
  // until the next report comes.
  it("holds the point let go of until a report made after the seek comes", async () => {
    const onCommand = jest.fn(answer);
    const { getByRole, getByText, rerender } = renderWithProviders(
      <LevelSoundMusicSection music={music()} onCommand={onCommand} />
    );

    getByRole("slider", { name: "Play the track from" }).focus();
    await userEvent.keyboard("{End}");

    expect(getByText("3:00 / 3:00")).toBeInTheDocument();

    rerender(<LevelSoundMusicSection music={music({ position: 65_400 })} onCommand={onCommand} />);

    expect(getByText("3:00 / 3:00")).toBeInTheDocument();

    rerender(<LevelSoundMusicSection music={music({ position: 179_600 })} onCommand={onCommand} />);

    expect(getByText("2:59 / 3:00")).toBeInTheDocument();
  });

  it("says when the next track is picked between tracks, and when the level has none", () => {
    const { getByText, rerender } = renderWithProviders(
      <LevelSoundMusicSection music={music({ track: null, wait: 9_200 })} onCommand={jest.fn(answer)} />
    );

    expect(getByText("In 10 s")).toBeInTheDocument();

    rerender(<LevelSoundMusicSection music={null} onCommand={jest.fn(answer)} />);

    expect(getByText("The level names no music.")).toBeInTheDocument();
  });
});
