import { describe, expect, it } from "@jest/globals";
import { DataGrid } from "@mui/x-data-grid";
import { render } from "@testing-library/react";

import { flagsColumn } from "@/core/ui/table/columns";

describe("flagsColumn", () => {
  it("sorts flags numerically while displaying uppercase hexadecimal", async () => {
    const { findAllByRole } = render(
      <DataGrid
        disableVirtualization
        columns={[flagsColumn("flags", "Flags")]}
        initialState={{ sorting: { sortModel: [{ field: "flags", sort: "asc" }] } }}
        rows={[
          { id: 1, flags: 16 },
          { id: 2, flags: 255 },
          { id: 3, flags: 2 },
          { id: 4, flags: 0x80000000 },
        ]}
      />
    );

    expect((await findAllByRole("gridcell")).map((cell) => cell.textContent)).toEqual([
      "0x2",
      "0x10",
      "0xFF",
      "0x80000000",
    ]);
  });

  it("displays zero flags while leaving missing values empty", async () => {
    const { findAllByRole } = render(
      <DataGrid
        disableVirtualization
        columns={[flagsColumn("flags", "Flags")]}
        rows={[{ id: 1, flags: null }, { id: 2, flags: 0 }, { id: 3 }]}
      />
    );

    expect((await findAllByRole("gridcell")).map((cell) => cell.textContent)).toEqual(["", "0x0", ""]);
  });
});
