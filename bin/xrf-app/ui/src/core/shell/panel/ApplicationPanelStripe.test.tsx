import { describe, expect, it, jest } from "@jest/globals";
import { userEvent } from "@testing-library/user-event";

import { IEditorPanel } from "@/core/shell/editor-shell";
import { ApplicationPanelStripe } from "@/core/shell/panel/ApplicationPanelStripe";
import { renderWithProviders } from "@/fixtures/utils/render";

const PANELS: Array<IEditorPanel> = [
  { id: "header", label: "Header", icon: <span>h</span>, render: () => <div>header panel</div> },
  { id: "bones", label: "Bones", icon: <span>b</span>, render: () => <div>bones panel</div> },
];

describe("ApplicationPanelStripe", () => {
  it("offers one control per declared panel", () => {
    const { getByLabelText } = renderWithProviders(
      <ApplicationPanelStripe side={"right"} panels={PANELS} activePanelId={null} onTogglePanel={jest.fn()} />
    );

    expect(getByLabelText("Header")).toBeInTheDocument();
    expect(getByLabelText("Bones")).toBeInTheDocument();
  });

  it("marks which panel is open", () => {
    const { getByLabelText } = renderWithProviders(
      <ApplicationPanelStripe side={"right"} panels={PANELS} activePanelId={"bones"} onTogglePanel={jest.fn()} />
    );

    expect(getByLabelText("Bones")).toHaveAttribute("aria-pressed", "true");
    expect(getByLabelText("Header")).toHaveAttribute("aria-pressed", "false");
  });

  it("reports the panel that was clicked", async () => {
    const onTogglePanel = jest.fn();

    const { getByLabelText } = renderWithProviders(
      <ApplicationPanelStripe side={"right"} panels={PANELS} activePanelId={"header"} onTogglePanel={onTogglePanel} />
    );

    await userEvent.click(getByLabelText("Bones"));

    expect(onTogglePanel).toHaveBeenCalledWith("bones");
  });

  it("reports the open panel again when it is clicked, which is how it collapses", async () => {
    const onTogglePanel = jest.fn();

    const { getByLabelText } = renderWithProviders(
      <ApplicationPanelStripe side={"right"} panels={PANELS} activePanelId={"header"} onTogglePanel={onTogglePanel} />
    );

    await userEvent.click(getByLabelText("Header"));

    expect(onTogglePanel).toHaveBeenCalledWith("header");
  });

  it("keeps the footer last, below whatever the application declared", () => {
    const { container } = renderWithProviders(
      <ApplicationPanelStripe
        side={"left"}
        panels={PANELS}
        activePanelId={null}
        footer={<span>settings</span>}
        onTogglePanel={jest.fn()}
      />
    );

    const stripe: Element = container.firstElementChild!;
    const order: Array<string> = Array.from(stripe.querySelectorAll("span, button"))
      .map((it: Element) => it.textContent ?? "")
      .filter((it: string) => it.length > 0);

    // What the shell owns sits under what the application published, on both sides.
    expect(order.at(-1)).toBe("settings");
  });

  it("stays present when an application declares nothing, so the frame does not shift", () => {
    const { container } = renderWithProviders(
      <ApplicationPanelStripe side={"right"} panels={[]} activePanelId={null} onTogglePanel={jest.fn()} />
    );

    expect(container.firstElementChild).toBeInTheDocument();
    expect(container.querySelectorAll("button")).toHaveLength(0);
  });
});
