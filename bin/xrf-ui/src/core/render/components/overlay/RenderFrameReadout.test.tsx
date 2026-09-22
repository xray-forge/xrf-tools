import { describe, expect, it } from "@jest/globals";
import { render } from "@testing-library/react";
import { EMPTY_RENDER_FRAME_COST, IRenderFrameCost } from "@xrf/renderer";

import { RenderFrameReadout } from "@/core/render/components/overlay/RenderFrameReadout";

const COST: IRenderFrameCost = {
  ...EMPTY_RENDER_FRAME_COST,
  drawnHeight: 1930,
  drawnWidth: 3217,
  draws: 886,
  frameTime: 6.25,
  framesPerSecond: 160,
  triangles: 6445460,
};

describe("RenderFrameReadout", () => {
  it("reads out what the frame cost and the size it was paid over", () => {
    const { getByText } = render(<RenderFrameReadout cost={COST} isOffscreen={false} />);

    expect(getByText("160 fps · 6.3 ms")).toBeInTheDocument();
    expect(getByText("886 draws · 6,445,460 tris")).toBeInTheDocument();
  });

  // The picture is identical either way, so the readout is the only place the answer can be seen.
  it.each([
    [false, "3217 × 1930 · main thread"],
    [true, "3217 × 1930 · worker"],
  ])("names where it was drawn, offscreen: %s", (isOffscreen: boolean, expected: string) => {
    const { getByText } = render(<RenderFrameReadout cost={COST} isOffscreen={isOffscreen} />);

    expect(getByText(expected)).toBeInTheDocument();
  });

  it("carries what only the scene can say, under the rest", () => {
    const { getByText } = render(
      <RenderFrameReadout cost={COST} isOffscreen={true}>
        <div>118 sectors · 138 MB</div>
      </RenderFrameReadout>
    );

    expect(getByText("118 sectors · 138 MB")).toBeInTheDocument();
  });

  it("reads out nothing drawn rather than nothing at all", () => {
    const { getByText } = render(<RenderFrameReadout cost={EMPTY_RENDER_FRAME_COST} isOffscreen={false} />);

    expect(getByText("0 fps · 0.0 ms")).toBeInTheDocument();
  });
});
