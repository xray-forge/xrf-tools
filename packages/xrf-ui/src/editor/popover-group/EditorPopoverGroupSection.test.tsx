import { describe, expect, it, jest } from "@jest/globals";
import { userEvent } from "@testing-library/user-event";

import { EditorIconAction } from "#/editor/icon-action";
import { ContentCopyIcon } from "#/icons/content-copy";
import { renderWithTheme } from "#/testing/render";

import { EditorPopoverGroupSection } from "./EditorPopoverGroupSection";

describe("EditorPopoverGroupSection", () => {
  it("ends its heading with what acts on the section, before its reset", async () => {
    const onCopy = jest.fn();
    const { getByRole } = renderWithTheme(
      <EditorPopoverGroupSection
        label={"Time"}
        actions={
          <EditorIconAction
            label={"Copy as LTX"}
            description={"Copies it"}
            icon={<ContentCopyIcon />}
            onClick={onCopy}
          />
        }
        resetLabel={"Reset to the weather"}
        onReset={() => {}}
      >
        <p>Clock</p>
      </EditorPopoverGroupSection>
    );
    const section: HTMLElement = getByRole("group", { name: "Time" });
    const buttons: Array<string | null> = Array.from(section.querySelectorAll("button")).map((button) =>
      button.getAttribute("aria-label")
    );

    expect(buttons).toEqual(["Copy as LTX", "Time: Reset to the weather"]);

    await userEvent.click(getByRole("button", { name: "Copy as LTX" }));

    expect(onCopy).toHaveBeenCalledTimes(1);
  });
});
