import { describe, expect, it } from "@jest/globals";
import { userEvent } from "@testing-library/user-event";
import { ReactElement, useState } from "react";

import { useEditorStatus } from "@/core/shell/editor-shell";
import { ApplicationStatusBar } from "@/core/shell/footer/ApplicationStatusBar";
import { renderWithProviders } from "@/fixtures/utils/render";

function Publisher({ segments }: { segments: Array<string> }): ReactElement {
  useEditorStatus(segments);

  return <div>publisher</div>;
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
  it("shows a resting state when no editor publishes anything", () => {
    const { getByText } = renderWithProviders(<ApplicationStatusBar />);

    expect(getByText("Ready")).toBeInTheDocument();
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

  it("clears the status when the publishing editor unmounts", async () => {
    const { findByText, getByText } = renderWithProviders(
      <>
        <Toggle />
        <ApplicationStatusBar />
      </>
    );

    expect(await findByText("12 340 objects")).toBeInTheDocument();

    await userEvent.click(getByText("unmount"));

    expect(getByText("Ready")).toBeInTheDocument();
  });
});
