import { describe, expect, it } from "@jest/globals";
import { GridRowId } from "@mui/x-data-grid";
import { userEvent } from "@testing-library/user-event";

import { DataTable } from "@/core/ui/table/DataTable";
import { renderWithProviders } from "@/fixtures/utils/render";

interface IRow {
  name: string;
  section: string;
}

const ROWS: Array<IRow> = [
  { name: "esc_stalker_1", section: "stalker_novice" },
  { name: "esc_stalker_2", section: "stalker_veteran" },
  { name: "gar_dog_1", section: "dog_normal" },
];

function renderTable(rows: Array<IRow> = ROWS, onRowSelect?: (row: IRow) => void) {
  return renderWithProviders(
    <DataTable<IRow>
      columns={[
        { field: "name", headerName: "Name", flex: 1 },
        { field: "section", headerName: "Section", flex: 1 },
      ]}
      countNoun={"object"}
      emptyLabel={"Nothing here."}
      getRowId={(row: IRow): GridRowId => row.name}
      getSearchText={(row: IRow): string => `${row.name} ${row.section}`}
      rows={rows}
      onRowSelect={onRowSelect}
    />
  );
}

describe("DataTable", () => {
  it("says so plainly when there is nothing to show", () => {
    const { getByText, queryByRole } = renderTable([]);

    expect(getByText("Nothing here.")).toBeInTheDocument();
    // No grid and no filter when there is nothing to look through.
    expect(queryByRole("textbox")).not.toBeInTheDocument();
  });

  it("counts rows with the caller's noun", () => {
    const { getByText } = renderTable();

    expect(getByText("3 object(s)")).toBeInTheDocument();
  });

  it("narrows as you filter and says how many are left", async () => {
    const { getByPlaceholderText, getByText } = renderTable();

    await userEvent.type(getByPlaceholderText("Filter"), "dog");

    expect(getByText("1 of 3 object(s)")).toBeInTheDocument();
  });

  it("matches case insensitively, since identifiers are not typed exactly", async () => {
    const { getByPlaceholderText, getByText } = renderTable();

    await userEvent.type(getByPlaceholderText("Filter"), "STALKER_VETERAN");

    expect(getByText("1 of 3 object(s)")).toBeInTheDocument();
  });

  it("restores all rows and input focus when the named filter is cleared", async () => {
    const { getByRole, getByText } = renderTable();
    const filter = getByRole("textbox", { name: "Filter rows" });

    await userEvent.type(filter, "dog");

    expect(getByText("1 of 3 object(s)")).toBeInTheDocument();

    await userEvent.click(getByRole("button", { name: "Clear filter" }));

    expect(filter).toHaveValue("");
    expect(filter).toHaveFocus();
    expect(getByText("3 object(s)")).toBeInTheDocument();
  });

  it("reports an empty filter result rather than looking broken", async () => {
    const { getByPlaceholderText, getByText } = renderTable();

    await userEvent.type(getByPlaceholderText("Filter"), "nothing matches");

    expect(getByText("0 of 3 object(s)")).toBeInTheDocument();
  });

  it("offers no filter when the caller supplies no search text", () => {
    const { queryByPlaceholderText } = renderWithProviders(
      <DataTable<IRow>
        columns={[{ field: "name", headerName: "Name" }]}
        countNoun={"object"}
        emptyLabel={"Nothing here."}
        getRowId={(row: IRow): GridRowId => row.name}
        rows={ROWS}
      />
    );

    expect(queryByPlaceholderText("Filter")).not.toBeInTheDocument();
  });

  it("renders a grid once there is something in it", () => {
    const { container } = renderTable();

    expect(container.querySelector(".MuiDataGrid-root")).toBeInTheDocument();
  });
});
