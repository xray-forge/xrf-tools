import { describe, expect, it, jest } from "@jest/globals";
import { fireEvent, waitFor } from "@testing-library/react";
import { userEvent } from "@testing-library/user-event";

import { renderWithProviders } from "@/fixtures/utils/render";

import { MotionPlaybackRate } from "./MotionPlaybackRate";

describe("MotionPlaybackRate", () => {
  it("opens a named dialog from the keyboard and restores focus when dismissed", async () => {
    const { getByRole, findByRole, queryByRole, getByTestId } = renderWithProviders(
      <MotionPlaybackRate
        data-testid={"rate"}
        id={"track-rate"}
        className={"track-control"}
        fps={30}
        onChange={jest.fn()}
      />
    );
    const button: HTMLElement = getByRole("button", { name: "Playback rate" });

    expect(getByTestId("rate")).toBe(button);
    expect(button).toHaveAttribute("id", "track-rate");
    expect(button).toHaveClass("track-control");
    expect(button).toHaveAttribute("aria-expanded", "false");

    await userEvent.tab();
    await userEvent.keyboard("{Enter}");

    const dialog: HTMLElement = await findByRole("dialog", { name: "Playback rate" });

    expect(button).toHaveAttribute("aria-expanded", "true");
    expect(button).toHaveAttribute("aria-controls", dialog.id);

    await userEvent.keyboard("{Escape}");

    await waitFor(() => expect(queryByRole("dialog")).not.toBeInTheDocument());
    expect(button).toHaveFocus();
    expect(button).toHaveAttribute("aria-expanded", "false");
  });

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
});
