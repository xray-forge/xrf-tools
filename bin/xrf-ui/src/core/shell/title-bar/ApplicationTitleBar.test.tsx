import { describe, expect, it } from "@jest/globals";
import { act, waitFor } from "@testing-library/react";
import { userEvent } from "@testing-library/user-event";

import { ApplicationTitleBar } from "@/core/shell/title-bar/ApplicationTitleBar";
import { REVEAL_DELAY_MS } from "@/core/ui/layout/delayed-reveal";
import { mockAppWindow, setMockWindowMaximized } from "@/fixtures/mocks/tauri.mocks";
import { renderWithProviders } from "@/fixtures/utils/render";

describe("ApplicationTitleBar", () => {
  it("drives the host window from its controls", async () => {
    const { getByLabelText } = renderWithProviders(<ApplicationTitleBar />);

    await userEvent.click(getByLabelText("Minimize"));
    expect(mockAppWindow.minimize).toHaveBeenCalledTimes(1);

    await userEvent.click(getByLabelText("Close"));
    expect(mockAppWindow.close).toHaveBeenCalledTimes(1);
  });

  it("follows the window when it is maximized by something other than its own button", async () => {
    const { getByLabelText, queryByLabelText } = renderWithProviders(<ApplicationTitleBar />);

    await waitFor(() => expect(getByLabelText("Maximize")).toBeInTheDocument());

    // A snap gesture or a double click on the drag region reaches the bar only as a resize.
    act(() => setMockWindowMaximized(true));

    await waitFor(() => expect(getByLabelText("Restore down")).toBeInTheDocument());
    expect(queryByLabelText("Maximize")).not.toBeInTheDocument();
  });

  it("keeps the caption draggable outside the controls", () => {
    const { container, getByLabelText, getByRole } = renderWithProviders(<ApplicationTitleBar />);

    // Without a drag region the window has no way to be moved at all, the system frame being gone.
    // It has to be `deep` and not `true`, or only the bar's own padding would drag.
    expect(container.querySelector("#application-title-bar")).toHaveAttribute("data-tauri-drag-region", "deep");

    // Tauri stops walking at a clickable element that declares no region of its own, so the controls
    // are excluded by carrying no attribute. Adding one here would make the buttons drag the window.
    expect(getByLabelText("Close")).not.toHaveAttribute("data-tauri-drag-region");

    // The mark was an `<img>`, which tauri does not treat as clickable but the browser does treat as
    // draggable, so it had to carry `draggable="false"` or the image drag won over the window move.
    // Inlined there is no image to drag, and the svg declares no region, so the deep walk reaches it.
    expect(getByRole("img", { name: "XRF tools" }).tagName.toLowerCase()).toBe("svg");
    expect(getByRole("img", { name: "XRF tools" })).not.toHaveAttribute("data-tauri-drag-region");
  });

  it("identifies the window by icon rather than by repeating the name below it", () => {
    const { getByRole, queryByText } = renderWithProviders(<ApplicationTitleBar />);

    expect(getByRole("img", { name: "XRF tools" })).toBeInTheDocument();
    expect(queryByText("XRF tools")).not.toBeInTheDocument();
  });

  it("draws the one progress line for whatever the active application is running", () => {
    // Queried including hidden elements, or the delayed reveal below would make an unrendered bar and
    // a not-yet-revealed one look the same.
    const idle = renderWithProviders(<ApplicationTitleBar />);

    expect(idle.queryByRole("progressbar", { hidden: true })).not.toBeInTheDocument();

    idle.unmount();

    // Editors used to portal their own bar in beside the breadcrumb, where a fixed-height flex row
    // left it competing for width instead of sitting under the caption.
    const busy = renderWithProviders(<ApplicationTitleBar isBusy />);

    expect(busy.getByRole("progressbar", { hidden: true })).toBeInTheDocument();
  });

  it("holds the progress line back so fast commands do not flash one", () => {
    const { getByRole } = renderWithProviders(<ApplicationTitleBar isBusy />);

    // Most commands answer in tens of milliseconds. Appearing and vanishing inside that reads as a
    // glitch, so the bar waits the same delay every other loader in the application waits.
    const style: CSSStyleDeclaration = getComputedStyle(getByRole("progressbar", { hidden: true }));

    expect(style.visibility).toBe("hidden");
    expect(style.animationDelay).toBe(`${REVEAL_DELAY_MS}ms`);
  });

  it("reserves the space between the icon and the controls", () => {
    const { getByRole, getByLabelText } = renderWithProviders(<ApplicationTitleBar />);

    // The gap is a real element rather than a margin, so a menu bar can land in it without the
    // controls or the icon moving.
    const reserved: Element = getByRole("img", { name: "XRF tools" }).nextElementSibling as Element;

    expect(reserved).toBeInTheDocument();
    expect(reserved.contains(getByLabelText("Minimize"))).toBe(false);
  });
});
