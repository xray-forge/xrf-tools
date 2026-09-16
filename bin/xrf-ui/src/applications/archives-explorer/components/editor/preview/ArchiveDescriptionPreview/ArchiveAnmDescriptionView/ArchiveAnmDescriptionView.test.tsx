import { describe, expect, it } from "@jest/globals";
import { RenderResult } from "@testing-library/react";

import { ArchiveAnmDescription, EArchiveAnimationBehavior } from "@/core/ipc/types/xrf-app";
import { mockArchiveAnimationChannel, mockArchiveAnmDescription } from "@/fixtures/mocks/archive.mocks";
import { renderWithProviders } from "@/fixtures/utils/render";

import { ArchiveAnmDescriptionView } from "./ArchiveAnmDescriptionView";

function renderView(description: ArchiveAnmDescription = mockArchiveAnmDescription()): RenderResult {
  return renderWithProviders(<ArchiveAnmDescriptionView description={description} />);
}

describe("ArchiveAnmDescriptionView", () => {
  it("leads with how long the engine plays it, from a range that counts both its ends", () => {
    const { getByText } = renderView();

    expect(getByText("2.00 s")).toBeTruthy();
    expect(getByText("Frames 0 to 59 inclusive, which is 60 at 30 fps")).toBeTruthy();
  });

  it("says an animation carries no name of its own rather than showing an empty one", () => {
    const { getByText } = renderView();

    expect(getByText("Not declared")).toBeTruthy();
    expect(getByText(/loaded by path/)).toBeTruthy();
  });

  it("names the animation the file saved one under", () => {
    const { getByText } = renderView(mockArchiveAnmDescription({ name: "camera_shake" }));

    expect(getByText("camera_shake")).toBeTruthy();
  });

  it("says when the keys reach past the range the file declares", () => {
    // `camera_effects\\earthquake_00.anm` and 100 others: keys the engine never reaches, because playback stops at
    // the declared end.
    const { getByText } = renderView(mockArchiveAnmDescription({ durationSeconds: 4, keyedSeconds: 9.967 }));

    expect(getByText("Reaching 9.97 s, past the end of the declared range")).toBeTruthy();
  });

  it("says when the keys stop short of it", () => {
    // `camera_effects\\dream.anm`: every key at zero inside a range declaring three and a third seconds.
    const { getByText } = renderView(mockArchiveAnmDescription({ durationSeconds: 3.333, keyedSeconds: 0 }));

    expect(getByText("Reaching 0.00 s, short of the declared end")).toBeTruthy();
  });

  it("says nothing about the reach when the keys end where the range does", () => {
    const { getByText } = renderView();

    expect(getByText("Across the six channels below")).toBeTruthy();
  });

  it("lists every channel, including the ones nothing keys", () => {
    const { getByText, getAllByText } = renderView();

    expect(getByText("position x")).toBeTruthy();
    expect(getByText("rotation bank")).toBeTruthy();
    expect(getByText("No keys")).toBeTruthy();
    expect(getAllByText("9 keys, 0.00–2.00 s")).toHaveLength(4);
  });

  it("reads a single key as the one moment it sits at rather than as a span", () => {
    const { getByText } = renderView();

    expect(getByText("1 key at 0.00 s")).toBeTruthy();
  });

  it("qualifies a channel with the values it reaches and the curves it uses", () => {
    const { getByText } = renderView();

    expect(getByText("-0.125 to 0.250 · tcb, linear")).toBeTruthy();
  });

  it("stays quiet about a channel that only holds outside its keys, which is all any shipped one does", () => {
    const { queryByText } = renderView();

    expect(queryByText(/hold before/)).toBeNull();
  });

  it("says what a channel does outside its keys where it is not the constant hold", () => {
    const { getByText } = renderView(
      mockArchiveAnmDescription({
        channels: [
          mockArchiveAnimationChannel("position x", {
            behaviorAfter: { kind: EArchiveAnimationBehavior.REPEAT },
          }),
        ],
      })
    );

    expect(getByText(/hold before, repeat after/)).toBeTruthy();
  });
});
