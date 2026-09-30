import { describe, expect, it } from "@jest/globals";

import { renderWithProviders } from "@/fixtures/utils/render";

import { CommandResultPathList } from "./CommandResultPathList";

describe("CommandResultPathList", () => {
  it("lists every path under its column", () => {
    const { container } = renderWithProviders(
      <CommandResultPathList
        paths={["gamedata.db0", "gamedata.db1"]}
        column={"Volume"}
        emptyLabel={"No volumes were written."}
        searchPlaceholder={"Filter by volume"}
      />
    );

    expect(container.textContent).toContain("Volume");
    expect(container.textContent).toContain("gamedata.db1");
  });

  it("says so when there is nothing to list", () => {
    const { container } = renderWithProviders(
      <CommandResultPathList
        paths={[]}
        column={"Volume"}
        emptyLabel={"No volumes were written."}
        searchPlaceholder={"Filter by volume"}
      />
    );

    expect(container.textContent).toContain("No volumes were written.");
  });
});
