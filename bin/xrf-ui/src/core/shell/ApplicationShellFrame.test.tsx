import { beforeEach, describe, expect, it } from "@jest/globals";
import { RenderResult } from "@testing-library/react";
import { userEvent } from "@testing-library/user-event";
import { ReactElement, ReactNode } from "react";
import { useLocation, useNavigate } from "react-router-dom";

import { NotificationsService } from "@/core/notifications/services";
import { ApplicationShellFrame } from "@/core/shell/ApplicationShellFrame";
import { EditorPanelsProvider, useEditorPanels } from "@/core/shell/panel/context";
import { setMockInvokeResponses } from "@/fixtures/mocks/tauri.mocks";
import { renderWithProviders } from "@/fixtures/utils/render";

/** Stands in for an editor that publishes one default-open panel, which most of them do. */
function EditorWithPanel({ name }: { name: string }): ReactElement {
  useEditorPanels(
    () => [{ icon: <span>{name}</span>, id: name, label: name, render: () => <div>{name} panel</div> }],
    [name]
  );

  return <div>{name} editor</div>;
}

/** Stands in for an editor that browses on the left and inspects on the right, as the dialogs editor does. */
function EditorWithBothPanels(): ReactElement {
  useEditorPanels(
    () => [
      { icon: <span>t</span>, id: "tree", label: "Tree", render: () => <div>tree panel</div>, side: "left" },
      { icon: <span>i</span>, id: "inspector", label: "Inspector", render: () => <div>inspector panel</div> },
    ],
    []
  );

  return <div>both editor</div>;
}

function EditorWithRouter(): ReactElement {
  const { pathname } = useLocation();
  const navigate = useNavigate();

  const name: string = pathname === "/archives-explorer" ? "Bones" : "Header";

  useEditorPanels(
    () => [{ icon: <span>{name}</span>, id: name, label: name, render: () => <div>{name} panel</div> }],
    [name]
  );

  return (
    <>
      <div>{name} editor</div>
      <button onClick={() => navigate("/exports-explorer")}>Open another tool</button>
    </>
  );
}

function renderFrame(children: ReactNode, route: string = "/"): RenderResult {
  return renderWithProviders(
    <EditorPanelsProvider>
      <ApplicationShellFrame>{children}</ApplicationShellFrame>
    </EditorPanelsProvider>,
    { bindings: [NotificationsService], route }
  );
}

describe("ApplicationShellFrame", () => {
  beforeEach(() => {
    // The frame remembers which panel was open, so a leftover choice would decide the next test.
    window.localStorage.clear();
    // The narrowest window the app supports, where the ratio binds and the fixed maximum never did.
    Object.defineProperty(window, "innerWidth", { configurable: true, value: 900, writable: true });
  });

  it("offers the jobs listing beside the notification centre in dev mode", async () => {
    // Reachability rather than rendering: the panel is registered in one list and its button rendered from another, so
    // a panel can be perfectly correct and still have no way to open it. That is exactly how it shipped invisible.
    setMockInvokeResponses({ "plugin:jobs|list": [] });

    const { getByLabelText, findByText } = renderFrame(<div>bare editor</div>);

    await userEvent.click(getByLabelText("Jobs"));

    expect(await findByText(/Nothing is running/)).toBeInTheDocument();
  });

  it("hides the jobs listing outside dev mode, where it answers nothing a person needs", () => {
    window.localStorage.setItem("xrf-dev-mode", "false");

    const { queryByLabelText } = renderFrame(<div>bare editor</div>);

    expect(queryByLabelText("Jobs")).toBeNull();
    expect(queryByLabelText("Notifications")).toBeInTheDocument();
  });

  it("offers the notification centre even when the active editor declares no panels", () => {
    const { getByLabelText } = renderFrame(<div>bare editor</div>);

    expect(getByLabelText("Notifications")).toBeInTheDocument();
  });

  it("selects notifications over an editor's default-open panel", async () => {
    const { getByLabelText, getByText, queryByText } = renderFrame(<EditorWithPanel name={"Bones"} />);

    expect(getByText("Bones panel")).toBeInTheDocument();

    await userEvent.click(getByLabelText("Notifications"));

    expect(getByText(/Nothing has been reported yet/)).toBeInTheDocument();
    expect(queryByText("Bones panel")).not.toBeInTheDocument();
  });

  it("selects an editor panel when notifications is open", async () => {
    const { getByLabelText, getByText, queryByText } = renderFrame(<EditorWithPanel name={"Bones"} />);

    await userEvent.click(getByLabelText("Notifications"));
    await userEvent.click(getByLabelText("Bones"));

    expect(getByText("Bones panel")).toBeInTheDocument();
    expect(queryByText(/Nothing has been reported yet/)).not.toBeInTheDocument();
  });

  it("keeps notifications open when navigating to another tool", async () => {
    const { getByLabelText, getByText, queryByText } = renderFrame(<EditorWithRouter />, "/archives-explorer");

    await userEvent.click(getByLabelText("Notifications"));
    await userEvent.click(getByText("Open another tool"));

    expect(getByText("Header editor")).toBeInTheDocument();
    expect(getByText(/Nothing has been reported yet/)).toBeInTheDocument();
    expect(queryByText("Header panel")).not.toBeInTheDocument();
  });

  it("collapses the right slot when notifications is clicked again", async () => {
    const { getByLabelText, queryByText } = renderFrame(<EditorWithPanel name={"Bones"} />);

    await userEvent.click(getByLabelText("Notifications"));
    await userEvent.click(getByLabelText("Notifications"));

    expect(queryByText(/Nothing has been reported yet/)).not.toBeInTheDocument();
    expect(queryByText("Bones panel")).not.toBeInTheDocument();
  });

  it("renders a lone panel at the window budget rather than its stored width", () => {
    window.localStorage.setItem("xrf.panels.right.width", "640");

    const { getByTestId } = renderFrame(<EditorWithPanel name={"Bones"} />);

    expect(getByTestId("application-panel-slot-right")).toHaveStyle({ minWidth: "450px", width: "450px" });
  });

  it("splits the budget so two open panels leave the content room", () => {
    window.localStorage.setItem("xrf.panels.left.width", "640");
    window.localStorage.setItem("xrf.panels.right.width", "640");

    const { getByTestId } = renderFrame(<EditorWithBothPanels />);

    expect(getByTestId("application-panel-slot-left")).toHaveStyle({ width: "225px" });
    expect(getByTestId("application-panel-slot-right")).toHaveStyle({ width: "225px" });
  });

  it("returns the closed panel's share to the one still open", async () => {
    window.localStorage.setItem("xrf.panels.left.width", "640");
    window.localStorage.setItem("xrf.panels.right.width", "640");

    const { getByLabelText, getByTestId } = renderFrame(<EditorWithBothPanels />);

    await userEvent.click(getByLabelText("Inspector"));

    expect(getByTestId("application-panel-slot-left")).toHaveStyle({ width: "450px" });
  });

  it("restores the global right-panel selection", () => {
    window.localStorage.setItem("xrf.panels.right.global", "notifications");

    const { getByText, queryByText } = renderFrame(<EditorWithPanel name={"Bones"} />);

    expect(getByText(/Nothing has been reported yet/)).toBeInTheDocument();
    expect(queryByText("Bones panel")).not.toBeInTheDocument();
  });
});
