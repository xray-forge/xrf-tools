import { describe, expect, it, jest } from "@jest/globals";
import { userEvent } from "@testing-library/user-event";

import { EToolId, IToolHelp } from "@/core/routing/tool";
import { renderWithProviders } from "@/fixtures/utils/render";

import { ToolHelpContent } from "./ToolHelpContent";

describe("ToolHelpContent", () => {
  it("renders only the sections the entry actually has, in rubric order", () => {
    const help: IToolHelp = {
      summary: "What this is for.",
      workflow: ["First step", "Second step"],
      nuances: ["A `nuance` worth knowing"],
    };

    const { getByText, queryByText } = renderWithProviders(<ToolHelpContent help={help} />);

    expect(getByText("What this is for.")).toBeInTheDocument();
    expect(getByText("Typical workflow")).toBeInTheDocument();
    expect(getByText("Nuances")).toBeInTheDocument();

    // An empty section is omitted, not padded: the rubric protects brevity rather than demanding it.
    expect(queryByText("Limitations")).not.toBeInTheDocument();
    expect(queryByText("Related tools")).not.toBeInTheDocument();
  });

  it("resolves related tools to catalog identity and reports leaving through one", async () => {
    const onNavigated = jest.fn();

    const help: IToolHelp = {
      summary: "Summary.",
      relatedTools: [EToolId.ARCHIVES_UNPACKER],
    };

    const { getByText } = renderWithProviders(<ToolHelpContent help={help} onNavigated={onNavigated} />, {
      route: "/archives-explorer",
    });

    await userEvent.click(getByText("Archives unpacker"));

    expect(onNavigated).toHaveBeenCalledTimes(1);
  });

  it("drops planned related tools with the section rather than linking to a signpost", () => {
    const help: IToolHelp = {
      summary: "Summary.",
      relatedTools: [EToolId.CHARACTERS_EXPLORER],
    };

    const { queryByText } = renderWithProviders(<ToolHelpContent help={help} />);

    expect(queryByText("Related tools")).not.toBeInTheDocument();
    expect(queryByText("Characters explorer")).not.toBeInTheDocument();
  });
});
