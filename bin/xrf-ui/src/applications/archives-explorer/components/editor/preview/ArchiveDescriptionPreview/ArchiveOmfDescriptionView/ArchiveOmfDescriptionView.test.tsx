import { describe, expect, it } from "@jest/globals";
import { fireEvent, RenderResult } from "@testing-library/react";

import { ArchiveOmfDescription, EArchiveOmfTarget } from "@/core/ipc/types/xrf-app";
import { mockArchiveOmfDescription, mockArchiveOmfMotion } from "@/fixtures/mocks/archive.mocks";
import { renderWithProviders } from "@/fixtures/utils/render";

import { ArchiveOmfDescriptionView } from "./ArchiveOmfDescriptionView";

function renderView(description: ArchiveOmfDescription = mockArchiveOmfDescription()): RenderResult {
  return renderWithProviders(<ArchiveOmfDescriptionView description={description} />);
}

describe("ArchiveOmfDescriptionView", () => {
  it("leads with what the bank holds", () => {
    const { getByText } = renderView();

    expect(getByText("2 parts · 3 bones")).toBeTruthy();
    expect(getByText("3.53 s")).toBeTruthy();
  });

  it("states the falloff rule once for the bank rather than on every motion", () => {
    const { getByText, queryAllByText } = renderView();

    expect(getByText(/rewrites the falloff of any motion that is not an effect/)).toBeTruthy();
    expect(queryAllByText(/rewrites the falloff/)).toHaveLength(1);
  });

  it("shows the blend the engine uses rather than the one the file declares", () => {
    // The declared pair is 2 and 2; the engine widens the accrue by half again and replaces the falloff.
    const { getAllByText } = renderView();

    expect(getAllByText(/blend 3.00 in, 2.99 out/).length).toBeGreaterThan(0);
  });

  it("names the part a cycle plays on, which an index alone does not", () => {
    const { getByText } = renderView(
      mockArchiveOmfDescription({
        motions: [
          mockArchiveOmfMotion({
            target: { kind: EArchiveOmfTarget.PART, index: 1, name: "legs" },
          }),
        ],
      })
    );

    expect(getByText(/part legs/)).toBeTruthy();
  });

  it("names a playback speed only where it is not the sampled rate", () => {
    const { queryByText, getByText } = renderView(
      mockArchiveOmfDescription({
        motions: [
          mockArchiveOmfMotion(),
          mockArchiveOmfMotion({
            name: "norm_run_0",
            speed: { value: 1.5, declared: 1.5, isClamped: false },
            playbackSeconds: 1.02,
          }),
        ],
      })
    );

    expect(getByText(/plays in 1.02 s at speed 1.50/)).toBeTruthy();
    expect(queryByText(/at speed 1.00/)).toBeNull();
  });

  it("says what the engine reads beside a value its quantizer refused", () => {
    const { getByText } = renderView(
      mockArchiveOmfDescription({
        motions: [
          mockArchiveOmfMotion({
            speed: { value: 0, declared: -1, isClamped: true },
            playbackSeconds: null,
          }),
        ],
      })
    );

    expect(getByText(/at speed 0.00 \(declared -1.00\)/)).toBeTruthy();
  });

  it("filters the motions by name and says how many of them are left", () => {
    const { getByLabelText, getByText, queryByText } = renderView();

    expect(getByText("Motions (2)")).toBeTruthy();

    fireEvent.change(getByLabelText("Filter motions"), { target: { value: "walk" } });

    expect(getByText("Motions (1 of 2)")).toBeTruthy();
    expect(queryByText("norm_idle_0")).toBeNull();
    expect(getByText("norm_walk_0")).toBeTruthy();
  });

  it("carries a mark's own intervals, and says when it declares none", () => {
    const { getByText } = renderView(
      mockArchiveOmfDescription({
        motions: [
          mockArchiveOmfMotion({
            marks: [
              { name: "Left foot", intervals: [{ from: 0.1, to: 0.25 }] },
              { name: "Right foot", intervals: [] },
            ],
          }),
        ],
      })
    );

    expect(getByText(/Left foot 0.10–0.25/)).toBeTruthy();
    expect(getByText(/Right foot, no interval/)).toBeTruthy();
  });
});
