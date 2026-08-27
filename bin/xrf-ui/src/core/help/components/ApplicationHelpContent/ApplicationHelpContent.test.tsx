import { describe, expect, it, jest } from "@jest/globals";
import { userEvent } from "@testing-library/user-event";

import { ApplicationHelpContent } from "@/core/help/components/ApplicationHelpContent/ApplicationHelpContent";
import { EApplicationId, IApplicationHelp } from "@/core/routing/application";
import { renderWithProviders } from "@/fixtures/utils/render";

describe("ApplicationHelpContent", () => {
  it("renders only the sections the entry actually has, in rubric order", () => {
    const help: IApplicationHelp = {
      summary: "What this is for.",
      workflow: ["First step", "Second step"],
      nuances: ["A `nuance` worth knowing"],
    };

    const { getByText, queryByText } = renderWithProviders(<ApplicationHelpContent help={help} />);

    expect(getByText("What this is for.")).toBeInTheDocument();
    expect(getByText("Typical workflow")).toBeInTheDocument();
    expect(getByText("Nuances")).toBeInTheDocument();

    // An empty section is omitted, not padded: the rubric protects brevity rather than demanding it.
    expect(queryByText("Limitations")).not.toBeInTheDocument();
    expect(queryByText("Related tools")).not.toBeInTheDocument();
  });

  it("resolves related tools to catalog identity and reports leaving through one", async () => {
    const onNavigated = jest.fn();

    const help: IApplicationHelp = {
      summary: "Summary.",
      relatedTools: [EApplicationId.ARCHIVES_UNPACKER],
    };

    const { getByText } = renderWithProviders(<ApplicationHelpContent help={help} onNavigated={onNavigated} />, {
      route: "/archives-explorer",
    });

    await userEvent.click(getByText("Archives unpacker"));

    expect(onNavigated).toHaveBeenCalledTimes(1);
  });

  it("drops planned related tools with the section rather than linking to a signpost", () => {
    const help: IApplicationHelp = {
      summary: "Summary.",
      relatedTools: [EApplicationId.CONFIGS_EXPLORER],
    };

    const { queryByText } = renderWithProviders(<ApplicationHelpContent help={help} />);

    expect(queryByText("Related tools")).not.toBeInTheDocument();
    expect(queryByText("Configs explorer")).not.toBeInTheDocument();
  });
});
