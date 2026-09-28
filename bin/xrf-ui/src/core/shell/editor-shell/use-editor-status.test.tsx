import { describe, expect, it } from "@jest/globals";
import { within } from "@testing-library/react";
import { userEvent } from "@testing-library/user-event";
import { ReactElement, useState } from "react";

import { IEditorStatusSegment, TEditorStatusSegment, useEditorStatus } from "@/core/shell/editor-shell";
import { ApplicationStatusBar } from "@/core/shell/footer/ApplicationStatusBar";
import { renderWithProviders } from "@/fixtures/utils/render";

function Publisher({ segments }: { segments: Array<TEditorStatusSegment> }): ReactElement {
  useEditorStatus(segments);

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
      {isMounted ? <Publisher segments={["12 340 objects"]} /> : null}
      <button onClick={() => setMounted(false)}>unmount</button>
    </>
  );
}

describe("useEditorStatus", () => {
  it("renders nothing when no editor publishes anything", () => {
    const { getByTestId } = renderWithProviders(<ApplicationStatusBar />);

    expect(getByTestId("application-status-bar")).toBeEmptyDOMElement();
  });

  it("renders published segments verbatim", async () => {
    const { findByText, getByText } = renderWithProviders(
      <>
        <Publisher segments={["3 archives", "512 files"]} />
        <ApplicationStatusBar />
      </>
    );

    expect(await findByText("3 archives")).toBeInTheDocument();
    expect(getByText("512 files")).toBeInTheDocument();
  });

  it("keeps segments containing spaces intact", async () => {
    const { findByText } = renderWithProviders(
      <>
        <Publisher segments={["12 340 objects"]} />
        <ApplicationStatusBar />
      </>
    );

    expect(await findByText("12 340 objects")).toBeInTheDocument();
  });

  it("shows a segment's details when it is hovered", async () => {
    const { findByRole, findByText } = renderWithProviders(
      <>
        <Publisher segments={["3 sectors", memorySegment("350 MB", "90 MB")]} />
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
        <Publisher segments={["3 sectors", memorySegment("350 MB", "90 MB")]} />
        <ApplicationStatusBar />
      </>
    );

    await userEvent.hover(await findByText("Backend 350 MB"));

    const tooltip: HTMLElement = await findByRole("tooltip");

    rerender(
      <>
        <Publisher segments={["Streaming", "4 sectors", memorySegment("420 MB", "130 MB")]} />
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

  it("clears the status when the publishing editor unmounts", async () => {
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
