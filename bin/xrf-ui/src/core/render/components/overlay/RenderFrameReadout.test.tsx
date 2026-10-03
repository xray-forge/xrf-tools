import { describe, expect, it } from "@jest/globals";
import { render } from "@testing-library/react";

import { RenderFrameReport } from "@/core/ipc/types/xrf-renderer";
import { RenderFrameReadout } from "@/core/render/components/overlay/RenderFrameReadout";
import { EMPTY_RENDER_FRAME_REPORT, EMPTY_RENDER_STATIC_REPORT } from "@/core/render/lib/native/native-frame-report";

const REPORT: RenderFrameReport = {
  ...EMPTY_RENDER_FRAME_REPORT,
  frameTime: 6.25,
  framesPerSecond: 160,
  height: 1930,
  renderHeight: 1930,
  renderWidth: 3217,
  staticDraws: { ...EMPTY_RENDER_STATIC_REPORT, commands: 886, keptTriangles: 6445460 },
  width: 3217,
};

describe("RenderFrameReadout", () => {
  it("reads out what the frame cost and the size it was paid over", () => {
    const { getByText } = render(<RenderFrameReadout report={REPORT} />);

    expect(getByText("160 fps · 6.3 ms")).toBeInTheDocument();
    expect(getByText("886 draws · 6,445,460 tris")).toBeInTheDocument();
  });

  it("says the size the frame was drawn at, and what it was upscaled from", () => {
    const { getByText, rerender } = render(<RenderFrameReadout report={REPORT} />);

    expect(getByText("3217 × 1930")).toBeInTheDocument();

    rerender(<RenderFrameReadout report={{ ...REPORT, renderHeight: 965, renderWidth: 1609 }} />);

    expect(getByText("3217 × 1930 from 1609 × 965")).toBeInTheDocument();
  });

  it("carries what only the scene can say, under the rest", () => {
    const { getByText } = render(
      <RenderFrameReadout report={REPORT}>
        <div>118 sectors · 138 MB</div>
      </RenderFrameReadout>
    );

    expect(getByText("118 sectors · 138 MB")).toBeInTheDocument();
  });

  it("reads out nothing drawn rather than nothing at all", () => {
    const { getByText } = render(<RenderFrameReadout report={{ ...EMPTY_RENDER_FRAME_REPORT, frameTime: null }} />);

    expect(getByText("0 fps · 0.0 ms")).toBeInTheDocument();
  });

  it("lists every pass's GPU cost under the frame while passes are timed, and nothing while they are not", () => {
    const passes = [
      { gpuTime: 0.5, name: "gbuffer" },
      { gpuTime: 0.25, name: "antialias" },
    ];
    const timed = render(<RenderFrameReadout report={{ ...EMPTY_RENDER_FRAME_REPORT, isGpuTimed: true, passes }} />);

    expect(timed.getByTestId("render-frame-passes")).toHaveTextContent("GPU0.75 msgbuffer0.50antialias0.25");
    timed.unmount();

    const untimed = render(<RenderFrameReadout report={{ ...EMPTY_RENDER_FRAME_REPORT, passes }} />);

    expect(untimed.queryByTestId("render-frame-passes")).not.toBeInTheDocument();
  });
});
