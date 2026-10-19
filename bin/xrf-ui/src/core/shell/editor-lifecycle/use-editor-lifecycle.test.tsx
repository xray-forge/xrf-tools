import { describe, expect, it } from "@jest/globals";
import { userEvent } from "@testing-library/user-event";
import { ReactElement, useState } from "react";

import { EditorToolbar } from "@/core/shell/editor/EditorToolbar";
import { useEditorBusy } from "@/core/shell/editor-lifecycle";
import { renderWithProviders } from "@/fixtures/utils/render";
import { noop } from "@/lib/callbacks/noop";

const BACK: { name: string } = { name: "Back to Spawn editor" };

function Busy({ isBusy }: { isBusy: boolean }): ReactElement {
  useEditorBusy(isBusy);

  return <div>editor</div>;
}

function Unmountable(): ReactElement {
  const [isMounted, setMounted] = useState(true);

  return (
    <>
      {isMounted ? <Busy isBusy /> : null}
      <button onClick={() => setMounted(false)}>unmount</button>
    </>
  );
}

function renderToolbar(editor: ReactElement) {
  return renderWithProviders(
    <>
      {editor}
      <EditorToolbar onBack={noop} />
    </>,
    { route: "/spawn-editor" }
  );
}

describe("useEditorBusy", () => {
  it("leaves navigation available when nothing is running", () => {
    const { getByRole } = renderToolbar(<Busy isBusy={false} />);

    expect(getByRole("button", BACK)).not.toBeDisabled();
  });

  it("blocks navigation while a command is running", () => {
    const { getByRole } = renderToolbar(<Busy isBusy />);

    // Walking away used to leave the command running against a screen nobody could see.
    expect(getByRole("button", BACK)).toBeDisabled();
  });

  it("releases the block when the editor unmounts, so a crash cannot strand the application", async () => {
    const { getByRole, getByText } = renderToolbar(<Unmountable />);

    expect(getByRole("button", BACK)).toBeDisabled();

    await userEvent.click(getByText("unmount"));

    expect(getByRole("button", BACK)).not.toBeDisabled();
  });
});
