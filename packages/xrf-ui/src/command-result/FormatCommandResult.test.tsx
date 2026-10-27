import { describe, expect, it } from "@jest/globals";

import { FormatCommandResult, IFormatCommandNouns, IFormatCommandOutcome } from "./FormatCommandResult";

import { renderWithTheme } from "#/testing/render";

const NOUNS: IFormatCommandNouns = { column: "Source", everything: "translation sources", item: "source" };

function outcome(patch: Partial<IFormatCommandOutcome> = {}): IFormatCommandOutcome {
  return {
    duration: 1200,
    invalidFiles: 0,
    startupDuration: 200,
    toFormat: [],
    totalFiles: 34,
    validFiles: 34,
    ...patch,
  };
}

describe("FormatCommandResult", () => {
  it("says a clean run found nothing, in the tool's own nouns", () => {
    const { container } = renderWithTheme(<FormatCommandResult isCheck nouns={NOUNS} result={outcome()} />);

    expect(container.textContent).toContain("All translation sources are correctly formatted");
    expect(container.textContent).toContain("sources");
  });

  it("reads the same number as a failure in a check and as work done in a rewrite", () => {
    const found: IFormatCommandOutcome = outcome({ invalidFiles: 2, toFormat: ["a.json", "b.json"], validFiles: 32 });

    expect(
      renderWithTheme(<FormatCommandResult isCheck nouns={NOUNS} result={found} />).container.textContent
    ).toContain("2 source(s) are not correctly formatted");
    expect(
      renderWithTheme(<FormatCommandResult isCheck={false} nouns={NOUNS} result={found} />).container.textContent
    ).toContain("Formatted 2 source(s)");
  });
});
