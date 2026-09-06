import { describe, expect, it } from "@jest/globals";

import { renderWithProviders } from "@/fixtures/utils/render";

import { EditorProblemsPanel, IEditorProblem } from "./EditorProblemsPanel";

const EMPTY_DESCRIPTION: string = "Every file in this project read cleanly.";

describe("EditorProblemsPanel", () => {
  it.each([
    { subject: "configs\\gameplay\\dialogs.xml", label: "dialogs.xml" },
    { subject: "configs/text/eng/strings.xml", label: "strings.xml" },
    { subject: "C:\\game/configs\\text/strings.xml", label: "strings.xml" },
    { subject: "missing_translation", label: "missing_translation" },
    { subject: "configs/", label: "configs/" },
  ])("shortens $subject without losing its original spelling", ({ subject, label }) => {
    const { getByTitle, getByText } = renderWithProviders(
      <EditorProblemsPanel
        findings={[{ rule: "dialog.missing", subject, message: "A referenced entry is missing." }]}
        rulePrefix={"dialog."}
        emptyDescription={EMPTY_DESCRIPTION}
      />
    );

    expect(getByTitle(subject).textContent).toBe(label);
    expect(getByTitle("dialog.missing").textContent).toBe("missing");
    expect(getByText("A referenced entry is missing.")).toBeInTheDocument();
  });

  it.each([null, ""])("renders a finding without a subject: %s", (subject) => {
    const { getByText, container } = renderWithProviders(
      <EditorProblemsPanel
        findings={[{ rule: "read.failed", subject, message: "Could not read the project." }]}
        emptyDescription={EMPTY_DESCRIPTION}
      />
    );

    expect(getByText("read.failed")).toBeInTheDocument();
    expect(getByText("Could not read the project.")).toBeInTheDocument();
    expect(container.querySelectorAll("[title]")).toHaveLength(1);
  });

  it("removes only a leading rule namespace and preserves repeated findings", () => {
    const finding: IEditorProblem = { rule: "other.dialog.missing", subject: null, message: "Missing entry." };
    const { getAllByText, getAllByRole } = renderWithProviders(
      <EditorProblemsPanel findings={[finding, finding]} rulePrefix={"dialog."} emptyDescription={EMPTY_DESCRIPTION} />
    );

    expect(getAllByText("other.dialog.missing")).toHaveLength(2);
    expect(getAllByText("Missing entry.")).toHaveLength(2);
    expect(getAllByRole("listitem")).toHaveLength(2);
  });

  it("keeps the panel identity and application copy when no problems are present", () => {
    const { getByTestId, getByText, queryByRole } = renderWithProviders(
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
    expect(queryByRole("list")).not.toBeInTheDocument();
  });
});
