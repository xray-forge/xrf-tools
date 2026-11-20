import { describe, expect, it, jest } from "@jest/globals";
import { userEvent } from "@testing-library/user-event";

import { renderWithTheme } from "#/testing/render";

import { EditorProblemsList, IEditorProblem } from "./EditorProblemsList";

const EMPTY_DESCRIPTION: string = "Every file in this project read cleanly.";

describe("EditorProblemsList", () => {
  it.each([
    { subject: "configs\\gameplay\\dialogs.xml", label: "dialogs.xml" },
    { subject: "configs/text/eng/strings.xml", label: "strings.xml" },
    { subject: "C:\\game/configs\\text/strings.xml", label: "strings.xml" },
    { subject: "missing_translation", label: "missing_translation" },
    { subject: "configs/", label: "configs/" },
  ])("shortens $subject without losing its original spelling", ({ subject, label }) => {
    const { getByTitle, getByText } = renderWithTheme(
      <EditorProblemsList
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
    const { getByText, container } = renderWithTheme(
      <EditorProblemsList
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
    const { getAllByText, getAllByRole } = renderWithTheme(
      <EditorProblemsList findings={[finding, finding]} rulePrefix={"dialog."} emptyDescription={EMPTY_DESCRIPTION} />
    );

    expect(getAllByText("other.dialog.missing")).toHaveLength(2);
    expect(getAllByText("Missing entry.")).toHaveLength(2);
    expect(getAllByRole("listitem")).toHaveLength(2);
  });

  it("states what was checked when nothing was found", () => {
    const { getByText, queryByRole } = renderWithTheme(
      <EditorProblemsList findings={[]} emptyDescription={EMPTY_DESCRIPTION} />
    );

    expect(getByText(`No problems found. ${EMPTY_DESCRIPTION}`)).toBeInTheDocument();
    expect(queryByRole("list")).not.toBeInTheDocument();
  });

  // Findings still being read are not a clean verdict, with or without some already in.
  it("says the findings are not all in, in place of the verdict that there are none", () => {
    const { getByTestId, queryByText, rerender } = renderWithTheme(
      <EditorProblemsList findings={[]} emptyDescription={EMPTY_DESCRIPTION} status={"Reading the rest…"} />
    );

    expect(getByTestId("editor-problems-status")).toHaveTextContent("Reading the rest…");
    expect(queryByText(`No problems found. ${EMPTY_DESCRIPTION}`)).not.toBeInTheDocument();

    rerender(
      <EditorProblemsList
        findings={[{ rule: "read.failed", subject: null, message: "Could not read the project." }]}
        emptyDescription={EMPTY_DESCRIPTION}
        status={"Reading the rest…"}
      />
    );

    expect(getByTestId("editor-problems-status")).toHaveTextContent("Reading the rest…");
    expect(queryByText("Could not read the project.")).toBeInTheDocument();
  });

  it("opens a finding that names a place, and leaves the others as plain rows", async () => {
    const onSelect = jest.fn();
    const { getAllByRole } = renderWithTheme(
      <EditorProblemsList
        findings={[
          { rule: "ltx.unknown", subject: "a.ltx", message: "Opens.", location: { path: "a.ltx", line: 4 } },
          { rule: "ltx.unknown", subject: "b.ltx", message: "Stays." },
        ]}
        emptyDescription={EMPTY_DESCRIPTION}
        onSelect={onSelect}
      />
    );

    const buttons: Array<HTMLElement> = getAllByRole("button");

    expect(buttons).toHaveLength(1);

    await userEvent.click(buttons[0]);

    expect(onSelect).toHaveBeenCalledWith({ path: "a.ltx", line: 4 });
  });
});
