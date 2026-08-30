import { describe, expect, it, jest } from "@jest/globals";
import { fireEvent } from "@testing-library/react";

import { MotionFrameSlider } from "@/core/visuals/components/preview/MotionFrameSlider";
import { renderWithProviders } from "@/fixtures/utils/render";

describe("MotionFrameSlider", () => {
  it("spans the frames a motion holds, counting the first as zero", () => {
    const { getByRole } = renderWithProviders(
      <MotionFrameSlider ariaLabel={"Motion frame"} frameCount={51} frame={20} onSeek={jest.fn()} />
    );
    const input: HTMLElement = getByRole("slider", { name: "Motion frame" });

    expect(input).toHaveAttribute("min", "0");
    // Fifty-one frames are numbered zero to fifty, and a slider reaching 51 would offer a frame nothing baked.
    expect(input).toHaveAttribute("max", "50");
    expect(input).toHaveValue("20");
  });

  it("has nothing to move while nothing is posed", () => {
    const { getByRole } = renderWithProviders(
      <MotionFrameSlider ariaLabel={"Motion frame"} frameCount={0} frame={0} onSeek={jest.fn()} />
    );

    expect(getByRole("slider", { name: "Motion frame" })).toBeDisabled();
  });

  it("reports the frame it was moved to", () => {
    const seeks: Array<number> = [];
    const { getByRole } = renderWithProviders(
      <MotionFrameSlider
        ariaLabel={"Motion frame"}
        frameCount={51}
        frame={20}
        onSeek={(frame: number) => seeks.push(frame)}
      />
    );

    fireEvent.change(getByRole("slider", { name: "Motion frame" }), { target: { value: "34" } });

    expect(seeks).toEqual([34]);
  });

  it("keeps the layout its caller poses beside the transition it owns", () => {
    // The two are merged as an array rather than spread, so a caller's own `sx` cannot drop the playhead rule and a
    // function or array `sx` survives.
    const { getByTestId } = renderWithProviders(
      <MotionFrameSlider
        ariaLabel={"Clip frame"}
        frameCount={10}
        frame={1}
        sx={{ marginX: 1, flexGrow: 1 }}
        onSeek={jest.fn()}
      />
    );

    expect(getByTestId("motion-frame-slider")).toHaveStyle({ flexGrow: 1 });
  });
});
