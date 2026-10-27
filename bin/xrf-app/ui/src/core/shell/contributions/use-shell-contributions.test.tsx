import { describe, expect, it } from "@jest/globals";
import { within } from "@testing-library/react";
import { userEvent } from "@testing-library/user-event";
import { Fragment, ReactElement, useState } from "react";

import { IShellContributed, useShellContributions, useShellPanels, useShellStatus } from "@/core/shell/contributions";
import { IEditorPanel, IEditorStatusSegment, TEditorStatusSegment } from "@/core/shell/editor-shell";
import { ApplicationStatusBar } from "@/core/shell/footer/ApplicationStatusBar";
import { renderWithProviders } from "@/fixtures/utils/render";

function DynamicPublisher(): ReactElement {
  const [label, setLabel] = useState<string>("First");

  useShellPanels(
    () => [
      {
        icon: <span>p</span>,
        id: "dynamic",
        label: "Dynamic",
        render: () => <div>{label}</div>,
      },
    ],
    [label]
  );

  return <button onClick={() => setLabel("Second")}>Update panel</button>;
}

/** Renders what the open tool publishes, and only that. */
function ToolPanelOutlet(): ReactElement {
  const { tool }: IShellContributed<IEditorPanel> = useShellContributions("panels");

  return (
    <>
      {tool.map((panel: IEditorPanel) => (
        <Fragment key={panel.id}>{panel.render()}</Fragment>
      ))}
    </>
  );
}

function StatusPublisher({ segments }: { segments: Array<TEditorStatusSegment> }): ReactElement {
  useShellStatus(segments);

  return <div>publisher</div>;
}

function memorySegment(backend: string, gpu: string): IEditorStatusSegment {
  return {
    id: "memory",
    text: `Backend ${backend}`,
    details: [
      { label: "Backend", value: backend },
      { label: "GPU", value: gpu, isNested: true },
    ],
  };
}

function Toggle(): ReactElement {
  const [isMounted, setMounted] = useState(true);

  return (
    <>
      {isMounted ? <StatusPublisher segments={["12 340 objects"]} /> : null}
      <button onClick={() => setMounted(false)}>unmount</button>
    </>
  );
}

describe("useShellPanels", () => {
  it("releases panels when their publisher unmounts", async () => {
    const { findByText, queryByText, rerender } = renderWithProviders(
      <>
        <DynamicPublisher />
        <ToolPanelOutlet />
      </>,
      { route: "/archives-explorer" }
    );

    expect(await findByText("First")).toBeInTheDocument();

    rerender(
      <>
        <ToolPanelOutlet />
      </>
    );

    expect(queryByText("First")).not.toBeInTheDocument();
  });

  it("publishes a new renderer when the panel array changes", async () => {
    const { findByText, getByText, queryByText } = renderWithProviders(
      <>
        <DynamicPublisher />
        <ToolPanelOutlet />
      </>,
      { route: "/archives-explorer" }
    );

    expect(await findByText("First")).toBeInTheDocument();

    await userEvent.click(getByText("Update panel"));

    expect(await findByText("Second")).toBeInTheDocument();
    expect(queryByText("First")).not.toBeInTheDocument();
  });

  it("publishes under the shell's key where no tool is open, never as a tool's", async () => {
    const { findByText, queryByText } = renderWithProviders(
      <>
        <DynamicPublisher />
        <ToolPanelOutlet />
      </>,
      { route: "/" }
    );

    // The fixture's own outlet draws left panels only, and this one goes on the right.
    expect(await findByText("Update panel")).toBeInTheDocument();
    expect(queryByText("First")).not.toBeInTheDocument();
  });
});

describe("useShellStatus", () => {
  it("renders nothing when nothing publishes", () => {
    const { getByTestId } = renderWithProviders(<ApplicationStatusBar />);

    expect(getByTestId("application-status-bar")).toBeEmptyDOMElement();
  });

  it("renders published segments verbatim", async () => {
    const { findByText, getByText } = renderWithProviders(
      <>
        <StatusPublisher segments={["3 archives", "512 files"]} />
        <ApplicationStatusBar />
      </>
    );

    expect(await findByText("3 archives")).toBeInTheDocument();
    expect(getByText("512 files")).toBeInTheDocument();
  });

  it("keeps segments containing spaces intact", async () => {
    const { findByText } = renderWithProviders(
      <>
        <StatusPublisher segments={["12 340 objects"]} />
        <ApplicationStatusBar />
      </>
    );

    expect(await findByText("12 340 objects")).toBeInTheDocument();
  });

  it("shows a segment's details when it is hovered", async () => {
    const { findByRole, findByText } = renderWithProviders(
      <>
        <StatusPublisher segments={["3 sectors", memorySegment("350 MB", "90 MB")]} />
        <ApplicationStatusBar />
      </>
    );

    await userEvent.hover(await findByText("Backend 350 MB"));

    const tooltip: HTMLElement = await findByRole("tooltip");

    expect(within(tooltip).getByText("Backend")).toBeInTheDocument();
    expect(within(tooltip).getByText("350 MB")).toBeInTheDocument();
    expect(within(tooltip).getByText("GPU")).toBeInTheDocument();
    expect(within(tooltip).getByText("90 MB")).toBeInTheDocument();
  });

  it("keeps a hovered segment's details open while its reading changes, and shows the new one", async () => {
    const { findByRole, findByText, getByText, rerender } = renderWithProviders(
      <>
        <StatusPublisher segments={["3 sectors", memorySegment("350 MB", "90 MB")]} />
        <ApplicationStatusBar />
      </>
    );

    await userEvent.hover(await findByText("Backend 350 MB"));

    const tooltip: HTMLElement = await findByRole("tooltip");

    rerender(
      <>
        <StatusPublisher segments={["Streaming", "4 sectors", memorySegment("420 MB", "130 MB")]} />
        <ApplicationStatusBar />
      </>
    );

    expect(getByText("Backend 420 MB")).toBeInTheDocument();
    expect(await findByRole("tooltip")).toBe(tooltip);
    expect(tooltip).toBeInTheDocument();
    expect(within(tooltip).getByText("420 MB")).toBeInTheDocument();
    expect(within(tooltip).getByText("130 MB")).toBeInTheDocument();
    expect(within(tooltip).queryByText("350 MB")).not.toBeInTheDocument();
  });

  it("clears the status when the publisher unmounts", async () => {
    const { findByText, getByTestId, getByText } = renderWithProviders(
      <>
        <Toggle />
        <ApplicationStatusBar />
      </>
    );

    expect(await findByText("12 340 objects")).toBeInTheDocument();

    await userEvent.click(getByText("unmount"));

    expect(getByTestId("application-status-bar")).toBeEmptyDOMElement();
  });
});
