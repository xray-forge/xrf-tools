import { describe, expect, it, jest } from "@jest/globals";
import { userEvent } from "@testing-library/user-event";
import { RefreshIcon } from "@xrf/ui/icons/refresh";
import { IconButton } from "@xrf/ui/material/icon-button";

import { EditorToolbar } from "@/core/shell/chrome/EditorToolbar";
import { renderWithProviders } from "@/fixtures/utils/render";

describe("EditorToolbar", () => {
  it("resolves its title from the route rather than a caller supplied string", () => {
    const { getByText } = renderWithProviders(<EditorToolbar />, { route: "/spawn-editor/alife" });

    expect(getByText("Spawn editor")).toBeInTheDocument();
  });

  it("names every tool the way the roster does, including nested routes", () => {
    const cases: Array<[string, string]> = [
      ["/archives-explorer", "Archives explorer"],
      ["/archives-unpacker", "Archives unpacker"],
      ["/sprite-equipment-editor", "Sprite equipment editor"],
      ["/visuals-explorer", "Visuals explorer"],
      ["/translations-editor", "Translations editor"],
    ];

    for (const [route, expected] of cases) {
      const { getByText, unmount } = renderWithProviders(<EditorToolbar />, { route });

      expect(getByText(expected)).toBeInTheDocument();

      // Renders share `document.body`, so each case is torn down before the next one asserts.
      unmount();
    }
  });

  it("names nothing and offers nothing to follow on a route owned by nothing", () => {
    const { queryByRole } = renderWithProviders(<EditorToolbar />, { route: "/nonsense" });

    // Going home is the menu's File › Close now; the breadcrumb names where you are and closes only what is open.
    expect(queryByRole("button")).not.toBeInTheDocument();
  });

  it("leaves the tool segment inert when nothing is open", () => {
    const { getByText, queryByRole } = renderWithProviders(<EditorToolbar />, { route: "/spawn-editor" });

    // A segment you can follow means there is something to close. The one arrow that did both could
    // never say that.
    expect(getByText("Spawn editor")).toBeInTheDocument();
    expect(queryByRole("button", { name: "Back to Spawn editor" })).not.toBeInTheDocument();
  });

  it("closes through the tool segment rather than through a button of its own", async () => {
    const onBack = jest.fn();

    const { getByLabelText } = renderWithProviders(<EditorToolbar onBack={onBack} />, { route: "/spawn-editor" });

    await userEvent.click(getByLabelText("Back to Spawn editor"));

    expect(onBack).toHaveBeenCalledTimes(1);
  });

  it("names the closing segment so its spoken name still contains what is written on it", () => {
    const { getByLabelText } = renderWithProviders(<EditorToolbar onBack={() => {}} />, { route: "/spawn-editor" });

    // A control whose spoken name does not include the word on it is one a voice user cannot ask for.
    expect(getByLabelText("Back to Spawn editor")).toHaveTextContent("Spawn editor");
  });

  it("divides its controls from the window's, and only when it has some", () => {
    const bare = renderWithProviders(<EditorToolbar />, { route: "/characters-explorer" });

    // With nothing to its left the rule was floating in empty space between the breadcrumb and the
    // window buttons, dividing nothing from nothing.
    expect(bare.queryByTestId("editor-toolbar-separator")).not.toBeInTheDocument();

    bare.unmount();

    const acting = renderWithProviders(<EditorToolbar actions={<button>refresh</button>} />, {
      route: "/characters-explorer",
    });

    expect(acting.getByTestId("editor-toolbar-separator")).toBeInTheDocument();
  });

  it("leaves help to the rail, so a tool with no actions of its own draws no group at all", () => {
    const { queryByLabelText, queryByTestId } = renderWithProviders(<EditorToolbar />, {
      route: "/archives-explorer",
    });

    expect(queryByLabelText("Help")).not.toBeInTheDocument();
    expect(queryByTestId("editor-toolbar-actions")).not.toBeInTheDocument();
    expect(queryByTestId("editor-toolbar-separator")).not.toBeInTheDocument();
  });

  it("sizes its controls against the caption rather than a toolbar that no longer exists", () => {
    const { getByLabelText } = renderWithProviders(
      <EditorToolbar
        actions={
          <IconButton aria-label={"Refresh"}>
            <RefreshIcon />
          </IconButton>
        }
      />,
      { route: "/spawn-editor" }
    );

    expect(getComputedStyle(getByLabelText("Refresh")).height).toBe("24px");
  });

  it("sets the path smaller than the names it follows", () => {
    const { getByText } = renderWithProviders(<EditorToolbar subtitle={"C:\\game\\all.spawn"} />, {
      route: "/spawn-editor",
    });

    const path: number = Number.parseFloat(getComputedStyle(getByText("C:\\game\\all.spawn")).fontSize);
    const name: number = Number.parseFloat(getComputedStyle(getByText("Spawn editor")).fontSize);

    // A path is context, not a heading. At the same size it competed with the tool it belongs to.
    expect(path).toBeLessThan(name);
  });

  it("renders the open document as the last breadcrumb segment", () => {
    const { getByText } = renderWithProviders(<EditorToolbar subtitle={"C:\\game\\all.spawn"} />, {
      route: "/spawn-editor",
    });

    expect(getByText("C:\\game\\all.spawn")).toBeInTheDocument();
    expect(getByText("Spawn editor")).toBeInTheDocument();
  });
});
