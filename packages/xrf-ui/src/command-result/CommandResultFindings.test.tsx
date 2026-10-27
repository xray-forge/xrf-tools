import { describe, expect, it } from "@jest/globals";
import { userEvent } from "@testing-library/user-event";

import { renderWithTheme } from "#/testing/render";

import { CommandResultFindings } from "./CommandResultFindings";

interface IRow {
  file: string;
}

const ROWS: Array<IRow> = [
  { file: "config\\system.ltx" },
  { file: "config\\weapons\\wpn_ak74.ltx" },
  { file: "config\\misc\\trade.ltx" },
];

function renderFindings(rows: Array<IRow>) {
  return renderWithTheme(
    <CommandResultFindings<IRow>
      rows={rows}
      columns={[{ field: "file", headerName: "File", flex: 1 }]}
      getRowId={(row) => row.file}
      getSearchText={(row) => row.file}
      emptyLabel={"Nothing to report."}
    />
  );
}

describe("CommandResultFindings", () => {
  it("says so plainly when a command found nothing", () => {
    const { getByText, queryByRole } = renderFindings([]);

    expect(getByText("Nothing to report.")).toBeInTheDocument();
    // No grid and no filter box when there is nothing to look through.
    expect(queryByRole("textbox")).not.toBeInTheDocument();
  });

  it("reports how many findings there are", () => {
    const { getByText } = renderFindings(ROWS);

    expect(getByText("3 finding(s)")).toBeInTheDocument();
  });

  it("narrows the findings as you filter, and says how many are left", async () => {
    const { getByPlaceholderText, getByText } = renderFindings(ROWS);

    await userEvent.click(getByPlaceholderText("Filter findings"));

    await userEvent.paste("wpn");

    expect(getByText("1 of 3 finding(s)")).toBeInTheDocument();
  });

  it("matches case insensitively, since paths are not typed exactly", async () => {
    const { getByPlaceholderText, getByText } = renderFindings(ROWS);

    await userEvent.click(getByPlaceholderText("Filter findings"));

    await userEvent.paste("WPN_AK74");

    expect(getByText("1 of 3 finding(s)")).toBeInTheDocument();
  });

  it("reports an empty filter result rather than looking broken", async () => {
    const { getByPlaceholderText, getByText } = renderFindings(ROWS);

    await userEvent.click(getByPlaceholderText("Filter findings"));

    await userEvent.paste("nothing matches this");

    expect(getByText("0 of 3 finding(s)")).toBeInTheDocument();
  });
});
