import { describe, expect, it, jest } from "@jest/globals";
import { waitFor } from "@testing-library/react";
import { userEvent } from "@testing-library/user-event";
import { ReactElement } from "react";

import { EditorSaver, useEditorDirty, useRequestLeave } from "@/core/shell/editor-lifecycle";
import { renderWithProviders } from "@/fixtures/utils/render";
import { Nullable } from "@/lib/types/general";

function Leaver({
  dirtyCount,
  onLeave,
  save = null,
}: {
  dirtyCount: number;
  onLeave: () => void;
  save?: Nullable<EditorSaver>;
}): ReactElement {
  const requestLeave: (leave: () => void) => void = useRequestLeave();

  useEditorDirty(dirtyCount, save);

  return (
    <button type={"button"} onClick={() => requestLeave(onLeave)}>
      Leave
    </button>
  );
}

describe("EditorLeaveDialog", () => {
  it("leaves immediately when nothing is unsaved", async () => {
    const onLeave = jest.fn();

    const { getByText } = renderWithProviders(
      <>
        <Leaver dirtyCount={0} onLeave={onLeave} />
      </>
    );

    await userEvent.click(getByText("Leave"));

    // No dialog is built at all for an editor holding nothing, which is every editor but this one.
    expect(onLeave).toHaveBeenCalledTimes(1);
  });

  it("asks before discarding unsaved work, and says how much", async () => {
    const onLeave = jest.fn();

    const { getByText } = renderWithProviders(
      <>
        <Leaver dirtyCount={3} onLeave={onLeave} />
      </>
    );

    await userEvent.click(getByText("Leave"));

    expect(getByText("Leave without saving?")).toBeInTheDocument();
    expect(getByText(/3 files have edits/)).toBeInTheDocument();
    expect(onLeave).not.toHaveBeenCalled();
  });

  it("stays put when the prompt is declined", async () => {
    const onLeave = jest.fn();

    const { getByText, queryByText } = renderWithProviders(
      <>
        <Leaver dirtyCount={1} onLeave={onLeave} />
      </>
    );

    await userEvent.click(getByText("Leave"));
    await userEvent.click(getByText("Stay"));

    expect(onLeave).not.toHaveBeenCalled();
    // The dialog animates out, so it is still in the tree for a frame after the click.
    await waitFor(() => expect(queryByText("Leave without saving?")).not.toBeInTheDocument());
  });

  it("goes through once the discard is confirmed", async () => {
    const onLeave = jest.fn();

    const { getByText } = renderWithProviders(
      <>
        <Leaver dirtyCount={1} onLeave={onLeave} />
      </>
    );

    await userEvent.click(getByText("Leave"));
    await userEvent.click(getByText("Discard and leave"));

    expect(onLeave).toHaveBeenCalledTimes(1);
  });

  it("phrases a single file as one", async () => {
    const { getByText } = renderWithProviders(
      <>
        <Leaver dirtyCount={1} onLeave={jest.fn()} />
      </>
    );

    await userEvent.click(getByText("Leave"));

    expect(getByText(/1 file has edits/)).toBeInTheDocument();
  });

  it("offers no save for an editor that published nothing to save with", async () => {
    // A file served out of an archive can be edited and read but has nowhere to be written, so the honest choice is
    // between discarding and staying rather than a button that would do nothing.
    const { getByText, queryByText } = renderWithProviders(
      <>
        <Leaver dirtyCount={1} onLeave={jest.fn()} />
      </>
    );

    await userEvent.click(getByText("Leave"));

    expect(queryByText("Save and leave")).not.toBeInTheDocument();
  });

  it("writes the work and then leaves", async () => {
    const onLeave = jest.fn();
    const save = jest.fn(async (): Promise<boolean> => true);

    const { getByText } = renderWithProviders(
      <>
        <Leaver dirtyCount={1} onLeave={onLeave} save={save} />
      </>
    );

    await userEvent.click(getByText("Leave"));
    await userEvent.click(getByText("Save and leave"));

    await waitFor(() => expect(onLeave).toHaveBeenCalledTimes(1));
    expect(save).toHaveBeenCalledTimes(1);
  });

  it("stays put when the save was refused", async () => {
    // Stale stamps, or a file taken away underneath the editor. Leaving anyway would discard the work the person
    // just asked to keep, and the notice explaining why is on the screen behind this dialog.
    const onLeave = jest.fn();
    const save = jest.fn(async (): Promise<boolean> => false);

    const { getByText } = renderWithProviders(
      <>
        <Leaver dirtyCount={1} onLeave={onLeave} save={save} />
      </>
    );

    await userEvent.click(getByText("Leave"));
    await userEvent.click(getByText("Save and leave"));

    await waitFor(() => expect(save).toHaveBeenCalledTimes(1));
    expect(onLeave).not.toHaveBeenCalled();
    expect(getByText("Leave without saving?")).toBeInTheDocument();
  });
});
