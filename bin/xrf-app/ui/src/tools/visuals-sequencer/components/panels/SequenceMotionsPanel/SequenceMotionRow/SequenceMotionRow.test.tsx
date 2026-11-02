import { describe, expect, it, jest } from "@jest/globals";
import { userEvent } from "@testing-library/user-event";

import { renderWithProviders } from "@/fixtures/utils/render";

import { SequenceMotionRow } from "./SequenceMotionRow";

describe("SequenceMotionRow", () => {
  it("requests another clip from the keyboard and displays the controlled usage count", async () => {
    const onAdd = jest.fn();
    const { getByRole, getByTestId, getByText, queryByText, rerender } = renderWithProviders(
      <SequenceMotionRow
        data-testid={"idle-row"}
        id={"idle-motion"}
        className={"available-motion"}
        motion={"idle"}
        usageCount={0}
        onAdd={onAdd}
      />
    );

    expect(getByTestId("idle-row")).toHaveAttribute("id", "idle-motion");
    expect(getByTestId("idle-row")).toHaveClass("available-motion");
    expect(queryByText("×0")).not.toBeInTheDocument();
    expect(getByRole("button", { name: "Add idle" })).toHaveAccessibleDescription("Add to the track");

    await userEvent.tab();
    await userEvent.keyboard("{Enter}");

    expect(onAdd).toHaveBeenCalledWith("idle");

    rerender(
      <>
        <SequenceMotionRow motion={"idle"} usageCount={2} onAdd={onAdd} />
      </>
    );

    expect(getByText("×2")).toBeInTheDocument();

    await userEvent.click(getByRole("button", { name: "Add idle" }));

    expect(onAdd).toHaveBeenCalledTimes(2);
    expect(getByText("×2")).toBeInTheDocument();
  });
});
