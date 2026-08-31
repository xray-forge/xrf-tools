import { describe, expect, it, jest } from "@jest/globals";
import { default as RefreshIcon } from "@mui/icons-material/Refresh";
import { IconButton } from "@mui/material";
import { userEvent } from "@testing-library/user-event";
import { Route, Routes } from "react-router-dom";

import { EditorToolbar } from "@/core/shell/editor/EditorToolbar";
import { renderWithProviders } from "@/fixtures/utils/render";

describe("EditorToolbar", () => {
  it("resolves its title from the route rather than a caller supplied string", () => {
    const { getByText } = renderWithProviders(<EditorToolbar />, { route: "/spawn-editor/alife" });

    expect(getByText("Spawn editor")).toBeInTheDocument();
  });

  it("names every application the way the roster does, including nested routes", () => {
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

  it("shows only the root on a route owned by nothing", () => {
    const { getByText, queryByText } = renderWithProviders(<EditorToolbar />, { route: "/nonsense" });

    expect(getByText("XRF")).toBeInTheDocument();
    expect(queryByText("Tools")).not.toBeInTheDocument();
  });

  it("goes home from the breadcrumb root", async () => {
    const { getByText, findByText } = renderWithProviders(
      <Routes>
        <Route path={"/spawn-editor"} element={<EditorToolbar />} />
        <Route path={"/"} element={<div>home</div>} />
      </Routes>,
      { route: "/spawn-editor" }
    );

    await userEvent.click(getByText("XRF"));

    expect(await findByText("home")).toBeInTheDocument();
  });

  it("leaves the application segment inert when nothing is open", () => {
    const { getByText, queryByRole } = renderWithProviders(<EditorToolbar />, { route: "/spawn-editor" });

    // A segment you can follow means there is something to close. The one arrow that did both could
    // never say that.
    expect(getByText("Spawn editor")).toBeInTheDocument();
    expect(queryByRole("button", { name: "Back to Spawn editor" })).not.toBeInTheDocument();
  });

  it("closes through the application segment rather than through a button of its own", async () => {
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
    const bare = renderWithProviders(<EditorToolbar />, { route: "/spawn-editor" });

    // With nothing to its left the rule was floating in empty space between the breadcrumb and the
    // window buttons, dividing nothing from nothing.
    expect(bare.container.querySelectorAll('[aria-hidden="true"]')).toHaveLength(1);

    bare.unmount();

    const acting = renderWithProviders(<EditorToolbar actions={<button>refresh</button>} />, {
      route: "/spawn-editor",
    });

    expect(acting.container.querySelectorAll('[aria-hidden="true"]')).toHaveLength(2);
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

    // A path is context, not a heading. At the same size it competed with the application it belongs to.
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
