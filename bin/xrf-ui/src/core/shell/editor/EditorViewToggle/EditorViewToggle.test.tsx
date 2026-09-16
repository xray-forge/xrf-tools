import { describe, expect, it, jest } from "@jest/globals";
import { default as RepeatIcon } from "@mui/icons-material/Repeat";
import { userEvent } from "@testing-library/user-event";

import { renderWithProviders } from "@/fixtures/utils/render";

import { EditorViewToggle } from "./EditorViewToggle";

describe("EditorViewToggle", () => {
  it("reports the controlled pressed state with a stable name", async () => {
    const onToggle = jest.fn();
    const { getByRole, getByTestId, rerender } = renderWithProviders(
      <EditorViewToggle
        data-testid={"loop"}
        id={"track-loop"}
        className={"track-control"}
        label={"Loop"}
        description={"Play once"}
        icon={<RepeatIcon />}
        isOn={false}
        onToggle={onToggle}
      />
    );
    const button: HTMLElement = getByRole("button", { name: "Loop", pressed: false });

    expect(getByTestId("loop")).toBe(button);
    expect(button).toHaveAttribute("id", "track-loop");
    expect(button).toHaveClass("track-control");
    expect(button).toHaveAccessibleDescription("Play once");

    await userEvent.tab();
    await userEvent.keyboard(" ");

    expect(onToggle).toHaveBeenCalledTimes(1);
    expect(button).toHaveAttribute("aria-pressed", "false");

    rerender(<EditorViewToggle label={"Loop"} icon={<RepeatIcon />} isOn={true} onToggle={onToggle} />);

    expect(getByRole("button", { name: "Loop", pressed: true })).toBeInTheDocument();
  });

  it("tells its three states apart, and never advertises one it cannot be acted on in", () => {
    function paintOf(isOn: boolean, isDisabled: boolean): string {
      const view = renderWithProviders(
        <EditorViewToggle
          label={"Loop"}
          icon={<RepeatIcon />}
          isOn={isOn}
          isDisabled={isDisabled}
          onToggle={() => {}}
        />
      );
      const style: CSSStyleDeclaration = getComputedStyle(view.getByRole("button", { name: "Loop" }));
      const paint: string = `${style.color} ${style.backgroundColor}`;

      view.unmount();

      return paint;
    }

    // Off used to be a 0.45 fade over a 0.38 one, which is the whole of what separated a view option somebody
    // had turned off from one this model could not offer at all.
    expect(paintOf(false, false)).not.toBe(paintOf(false, true));
    expect(paintOf(true, false)).not.toBe(paintOf(false, false));

    // Alpha, bump and skeleton each default on while being unavailable whenever the model carries neither, so a
    // disabled toggle that is nominally on is the ordinary case rather than the corner one.
    expect(paintOf(true, true)).toBe(paintOf(false, true));
  });

  it("keeps the unavailable reason reachable without activating the toggle", async () => {
    const onToggle = jest.fn();
    const { getByRole, getByTitle, findByRole } = renderWithProviders(
      <EditorViewToggle
        label={"Loop"}
        icon={<RepeatIcon />}
        isOn={false}
        isDisabled={true}
        unavailableTitle={"No motion available"}
        onToggle={onToggle}
      />
    );
    const button: HTMLElement = getByRole("button", { name: "Loop", pressed: false });
    const wrapper: HTMLElement = getByTitle("No motion available");

    expect(button).toBeDisabled();
    expect(button).toHaveAccessibleDescription("No motion available");

    await userEvent.hover(wrapper);

    expect(await findByRole("tooltip")).toHaveTextContent("No motion available");

    await userEvent.click(wrapper);

    expect(onToggle).not.toHaveBeenCalled();
  });
});
