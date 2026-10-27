import { describe, expect, it } from "@jest/globals";

import { CommandResultPathList } from "./CommandResultPathList";

import { renderWithTheme } from "#/testing/render";

describe("CommandResultPathList", () => {
  it("lists every path under its column", () => {
    const { container } = renderWithTheme(
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
    const { container } = renderWithTheme(
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
