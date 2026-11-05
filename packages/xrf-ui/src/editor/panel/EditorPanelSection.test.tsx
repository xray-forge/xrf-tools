import { describe, expect, it, jest } from "@jest/globals";
import { userEvent } from "@testing-library/user-event";

import { EditorIconAction } from "#/editor/icon-action";
import { PlayArrowIcon } from "#/icons/play-arrow";
import { renderWithTheme } from "#/testing/render";

import { EditorPanelSection } from "./EditorPanelSection";

describe("EditorPanelSection", () => {
  it("ends its title line with what acts on the whole group", async () => {
    const onPlay = jest.fn();
    const { getByRole } = renderWithTheme(
      <EditorPanelSection
        title={"Ambient effects"}
        actions={
          <EditorIconAction
            label={"Play one now"}
            description={"Starts one"}
            icon={<PlayArrowIcon />}
            onClick={onPlay}
          />
        }
      >
        <p>Playing</p>
      </EditorPanelSection>
    );
    const heading: HTMLElement = getByRole("heading", { name: "Ambient effects" });

    // The action shares the heading's line, after it.
    expect(heading.parentElement).toContainElement(getByRole("button", { name: "Play one now" }));

    await userEvent.click(getByRole("button", { name: "Play one now" }));

    expect(onPlay).toHaveBeenCalledTimes(1);
  });
});
