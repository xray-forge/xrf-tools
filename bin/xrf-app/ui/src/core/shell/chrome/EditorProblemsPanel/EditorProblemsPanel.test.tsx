import { describe, expect, it } from "@jest/globals";

import { renderWithProviders } from "@/fixtures/utils/render";

import { EditorProblemsPanel } from "./EditorProblemsPanel";

const EMPTY_DESCRIPTION: string = "Every file in this project read cleanly.";

describe("EditorProblemsPanel", () => {
  it("keeps the panel identity and tool copy when no problems are present", () => {
    const { getByTestId, getByText } = renderWithProviders(
      <EditorProblemsPanel
        data-testid={"problems"}
        id={"project-problems"}
        className={"project-panel"}
        findings={[]}
        emptyDescription={EMPTY_DESCRIPTION}
      />
    );

    expect(getByTestId("problems")).toHaveAttribute("id", "project-problems");
    expect(getByTestId("problems")).toHaveClass("project-panel");
    expect(getByText("Problems")).toBeInTheDocument();
    expect(getByText(`No problems found. ${EMPTY_DESCRIPTION}`)).toBeInTheDocument();
  });
});
