import { describe, expect, it } from "@jest/globals";
import { DataGrid } from "@mui/x-data-grid";
import { render } from "@testing-library/react";

import { decimalColumn, textColumn } from "@/core/ui/table/columns";

describe("decimalColumn sorting", () => {
  it("sorts decimal values numerically while displaying two decimal places", async () => {
    const { findAllByRole } = render(
      <DataGrid
        disableVirtualization
        columns={[decimalColumn("distance", "Distance")]}
        initialState={{ sorting: { sortModel: [{ field: "distance", sort: "asc" }] } }}
        rows={[
          { id: 1, distance: 10 },
          { id: 2, distance: 2 },
          { id: 3, distance: -1 },
          { id: 4, distance: 0 },
        ]}
      />
    );

    expect((await findAllByRole("gridcell")).map((cell) => cell.textContent)).toEqual([
      "-1.00",
      "0.00",
      "2.00",
      "10.00",
    ]);
  });

  it("sorts by full precision when different values have the same displayed text", async () => {
    const { findAllByRole } = render(
      <DataGrid
        disableVirtualization
        columns={[textColumn("id", "Name"), decimalColumn("distance", "Distance")]}
        initialState={{ sorting: { sortModel: [{ field: "distance", sort: "asc" }] } }}
        rows={[
          { id: "higher", distance: 1.234 },
          { id: "lower", distance: 1.231 },
        ]}
      />
    );

    expect((await findAllByRole("gridcell")).map((cell) => cell.textContent)).toEqual([
      "lower",
      "1.23",
      "higher",
      "1.23",
    ]);
  });
});
