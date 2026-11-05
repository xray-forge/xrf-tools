import { describe, expect, it, jest } from "@jest/globals";
import { fireEvent } from "@testing-library/react";
import { userEvent } from "@testing-library/user-event";

import { renderWithProviders } from "@/fixtures/utils/render";

import { MotionPlaybackRate } from "./MotionPlaybackRate";

describe("MotionPlaybackRate", () => {
  it("requests rate changes and reflects the rate supplied by its owner", async () => {
    const onChange = jest.fn();
    const { getByRole, findByRole, rerender } = renderWithProviders(
      <MotionPlaybackRate fps={30} onChange={onChange} />
    );

    await userEvent.click(getByRole("button", { name: "Playback rate" }));

    const slider: HTMLElement = await findByRole("slider", { name: "Frames a second" });

    expect(slider).toHaveAttribute("min", "1");
    expect(slider).toHaveAttribute("max", "120");

    fireEvent.change(slider, { target: { value: "60" } });

    expect(onChange).toHaveBeenCalledWith(60);
    expect(slider).toHaveAttribute("aria-valuenow", "30");

    rerender(
      <>
        <MotionPlaybackRate fps={60} onChange={onChange} />
      </>
    );

    expect(getByRole("slider", { name: "Frames a second" })).toHaveAttribute("aria-valuenow", "60");
  });

  it("sets the rate back to the motions' own from its heading, only while it is off it", async () => {
    const onChange = jest.fn();
    const { getByRole, queryByRole, rerender } = renderWithProviders(
      <MotionPlaybackRate fps={30} onChange={onChange} />
    );
    const reset: { name: RegExp } = { name: /^Playback rate: Reset to 30 fps/ };

    await userEvent.click(getByRole("button", { name: "Playback rate" }));

    expect(queryByRole("button", reset)).toBeNull();

    rerender(<MotionPlaybackRate fps={60} onChange={onChange} />);
    await userEvent.click(getByRole("button", reset));

    expect(onChange).toHaveBeenCalledWith(30);
  });
});
