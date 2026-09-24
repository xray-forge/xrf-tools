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
    const { getByText } = render(<RenderFrameReadout cost={COST} />);

    expect(getByText("160 fps · 6.3 ms")).toBeInTheDocument();
    expect(getByText("886 draws · 6,445,460 tris")).toBeInTheDocument();
  });

  it("says the size the frame was drawn at", () => {
    const { getByText } = render(<RenderFrameReadout cost={COST} />);

    expect(getByText("3217 × 1930")).toBeInTheDocument();
  });

  it("carries what only the scene can say, under the rest", () => {
    const { getByText } = render(
      <RenderFrameReadout cost={COST}>
        <div>118 sectors · 138 MB</div>
      </RenderFrameReadout>
    );

    expect(getByText("118 sectors · 138 MB")).toBeInTheDocument();
  });

  it("reads out nothing drawn rather than nothing at all", () => {
    const { getByText } = render(<RenderFrameReadout cost={EMPTY_RENDER_FRAME_COST} />);

    expect(getByText("0 fps · 0.0 ms")).toBeInTheDocument();
  });

  it("lists every pass's GPU cost under the frame while passes are timed, and nothing while they are not", () => {
    const passes = [
      { gpuTime: 0.5, name: "gbuffer" },
      { gpuTime: 0.25, name: "antialias" },
    ];
    const timed = render(<RenderFrameReadout cost={EMPTY_RENDER_FRAME_COST} timings={{ isGpuTimed: true, passes }} />);

    expect(timed.getByTestId("render-frame-passes")).toHaveTextContent("GPU0.75 msgbuffer0.50antialias0.25");
    timed.unmount();

    const untimed = render(
      <RenderFrameReadout cost={EMPTY_RENDER_FRAME_COST} timings={{ isGpuTimed: false, passes }} />
    );

    expect(untimed.queryByTestId("render-frame-passes")).not.toBeInTheDocument();
  });
});
