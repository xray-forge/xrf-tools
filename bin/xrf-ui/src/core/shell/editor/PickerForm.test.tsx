import { describe, expect, it } from "@jest/globals";
import { fireEvent } from "@testing-library/react";

import { PickerForm } from "@/core/shell/editor/PickerForm";
import { renderWithProviders } from "@/fixtures/utils/render";

describe("PickerForm", () => {
  it("carries the standard toolbar so the frame does not change between a form and a workspace", () => {
    const { getByText } = renderWithProviders(<PickerForm title={"Provide spawn file to open"} />, {
      route: "/spawn-editor",
    });

    expect(getByText("Spawn editor")).toBeInTheDocument();
    expect(getByText("Provide spawn file to open")).toBeInTheDocument();
  });

  it("leaves through the breadcrumb root, rather than carrying a back button of its own", () => {
    const { getByText, queryByLabelText } = renderWithProviders(<PickerForm title={"Open"} />, {
      route: "/spawn-editor",
    });

    expect(getByText("XRF")).toBeInTheDocument();
    expect(queryByLabelText("Close document")).not.toBeInTheDocument();
  });

  it("stops the way out while an operation is in flight", () => {
    const { getByText } = renderWithProviders(<PickerForm title={"Open"} isLoading />, { route: "/spawn-editor" });

    // Disabled rather than removed: a control that vanishes mid-operation is harder to trust.
    expect(getByText("XRF")).toBeDisabled();
  });

  it("draws no progress of its own, leaving the one running command to the caption band", () => {
    const busy = renderWithProviders(<PickerForm title={"Open"} isLoading />, { route: "/spawn-editor" });

    // The form publishes `isLoading` through `useEditorBusy`, and the shell draws it once. Drawing it
    // here too put two bars on screen for one command.
    expect(busy.queryByRole("progressbar")).not.toBeInTheDocument();
  });

  it("surfaces an error without hiding the form", () => {
    const { getByText } = renderWithProviders(<PickerForm title={"Open"} error={"Failed to read file"} />, {
      route: "/spawn-editor",
    });

    expect(getByText("Failed to read file")).toBeInTheDocument();
    expect(getByText("Open")).toBeInTheDocument();
  });

  it("says what the command touches before it is run", () => {
    const { getByText } = renderWithProviders(
      <PickerForm title={"Unpack"} description={"Writes the chunks into the destination directory."} />,
      { route: "/spawn-unpacker" }
    );

    expect(getByText("Writes the chunks into the destination directory.")).toBeInTheDocument();
  });

  it("keeps the actions with the parameters they act on", () => {
    const { getByText } = renderWithProviders(<PickerForm title={"Open"} submitLabel={"Open file"} />, {
      route: "/spawn-editor",
    });

    // Both buttons belong to the one panel, rather than the form floating at the top of a window whose
    // bottom edge holds the buttons.
    const panel: HTMLElement | null = getByText("Open").closest(".MuiPaper-root");

    expect(panel).toContainElement(getByText("Open file"));
    expect(panel).toContainElement(getByText("Back"));
  });

  it("renders a result alongside the form", () => {
    const { getByText } = renderWithProviders(<PickerForm title={"Unpack"} result={<div>unpacked 512 files</div>} />, {
      route: "/archives-unpacker",
    });

    expect(getByText("unpacked 512 files")).toBeInTheDocument();
  });

  // The parameters used to keep 55% of the window and scroll inside it, which gave the screen two
  // stacked scrolling regions. They fold away instead, so the result gets the room and the only thing
  // that scrolls is whatever the result puts there.
  it("folds the parameters away once there is a result to read", () => {
    const { queryByText, getByText } = renderWithProviders(
      <PickerForm title={"Unpack"} result={<div>unpacked 512 files</div>}>
        <div>source directory</div>
      </PickerForm>,
      { route: "/archives-unpacker" }
    );

    expect(getByText("unpacked 512 files")).toBeInTheDocument();
    expect(queryByText("source directory")).not.toBeInTheDocument();
  });

  // Opening them is a statement of intent - "I am still working on these" - and it has to outlast the
  // next run, or adjusting a path and re-running costs a click to reopen the form every single time.
  it("leaves the parameters open once they have been opened by hand", () => {
    const { getByLabelText, getByText, rerender } = renderWithProviders(
      <PickerForm title={"Unpack"} result={<div>first run</div>}>
        <div>source directory</div>
      </PickerForm>,
      { route: "/archives-unpacker" }
    );

    fireEvent.click(getByLabelText("Show parameters"));

    expect(getByText("source directory")).toBeInTheDocument();

    // Every screen clears its result when a run starts, so a second run is not one prop change but
    // two. It is that round trip through "no result" which re-arms the fold, and which a test holding
    // the result present throughout would never exercise.
    rerender(
      <>
        <PickerForm title={"Unpack"}>
          <div>source directory</div>
        </PickerForm>
      </>
    );

    rerender(
      <>
        <PickerForm title={"Unpack"} result={<div>second run</div>}>
          <div>source directory</div>
        </PickerForm>
      </>
    );

    expect(getByText("second run")).toBeInTheDocument();
    expect(getByText("source directory")).toBeInTheDocument();
  });

  // The status sits outside the parameters, so folding cannot take it away. No screen pairs it with a
  // result today — the two spawn screens report a run this way and produce nothing else — so only this
  // test holds the two together.
  it("keeps the status visible while the parameters are folded away", () => {
    const { queryByText, getByText } = renderWithProviders(
      <PickerForm title={"Unpack"} status={<div>Archives unpacked.</div>} result={<div>unpacked 512 files</div>}>
        <div>source directory</div>
      </PickerForm>,
      { route: "/archives-unpacker" }
    );

    expect(queryByText("source directory")).not.toBeInTheDocument();
    expect(getByText("Archives unpacked.")).toBeInTheDocument();
  });
});
