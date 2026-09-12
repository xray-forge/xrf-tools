import { describe, expect, it, jest } from "@jest/globals";
import { RenderResult } from "@testing-library/react";
import { userEvent } from "@testing-library/user-event";
import { ReactElement, useState } from "react";

import { IUseRankedSearch, useRankedSearch } from "@/core/search/lib/use-ranked-search";
import { renderWithProviders } from "@/fixtures/utils/render";

interface IFile {
  name: string;
}

const FILES: Array<IFile> = [
  { name: "configs\\aaa_dialogs_old.xml" },
  { name: "dialogs.xml" },
  { name: "configs\\dialogs_zaton.xml" },
];

function toSearchText(file: IFile): string {
  return file.name;
}

function renderHarness(onSelect: (file: IFile) => void, limit?: number): RenderResult {
  function Harness(): ReactElement {
    const search: IUseRankedSearch<IFile> = useRankedSearch({ items: FILES, toSearchText, limit, onSelect });

    return (
      <div>
        <input
          aria-label={"search"}
          value={search.query}
          onChange={(event) => search.setQuery(event.target.value)}
          onKeyDown={search.onInputKeyDown}
        />
        <button onClick={search.clear}>clear</button>
        <div data-testid={"active"}>{search.activeIndex}</div>
        <div data-testid={"total"}>{search.total}</div>
        <ol>
          {search.results.map((result) => (
            <li key={result.item.name}>{result.item.name}</li>
          ))}
        </ol>
      </div>
    );
  }

  return renderWithProviders(<Harness />);
}

describe("useRankedSearch", () => {
  it.each(["primary", "secondary"])("reindexes when the %s text extractor changes", async (field) => {
    function Harness(): ReactElement {
      const [hasExtraText, setHasExtraText] = useState(false);
      const search: IUseRankedSearch<IFile> = useRankedSearch({
        items: FILES,
        toSearchText: hasExtraText && field === "primary" ? (file) => `extra ${file.name}` : toSearchText,
        toSecondaryText: hasExtraText && field === "secondary" ? () => "extra" : undefined,
      });

      return (
        <>
          <input aria-label={"search"} value={search.query} onChange={(event) => search.setQuery(event.target.value)} />
          <button onClick={() => setHasExtraText((current) => !current)}>Toggle extra text</button>
          <div data-testid={"total"}>{search.total}</div>
        </>
      );
    }

    const { getByRole, getByTestId } = renderWithProviders(<Harness />);
    const input: HTMLElement = getByRole("textbox", { name: "search" });

    await userEvent.type(input, "extra");

    expect(getByTestId("total")).toHaveTextContent("0");

    await userEvent.click(getByRole("button", { name: "Toggle extra text" }));

    expect(input).toHaveValue("extra");
    expect(getByTestId("total")).toHaveTextContent("3");

    await userEvent.click(getByRole("button", { name: "Toggle extra text" }));

    expect(getByTestId("total")).toHaveTextContent("0");
  });

  it("never renders a frame describing an older query for a small list", async () => {
    const rendered: Array<{ query: string; isStale: boolean; total: number }> = [];

    function Harness(): ReactElement {
      const search: IUseRankedSearch<IFile> = useRankedSearch({ items: FILES, toSearchText });

      rendered.push({ query: search.query, isStale: search.isStale, total: search.total });

      return (
        <input aria-label={"search"} value={search.query} onChange={(event) => search.setQuery(event.target.value)} />
      );
    }

    const { getByLabelText } = renderWithProviders(<Harness />);

    // Nonsense, because a query that matches nothing is where a stale frame used to show the unfiltered list.
    await userEvent.type(getByLabelText("search"), "qqq");

    expect(rendered.filter((frame) => frame.isStale)).toHaveLength(0);
    expect(rendered.filter((frame) => frame.query !== "" && frame.total !== 0)).toHaveLength(0);
  });

  it("returns nothing until something is typed", () => {
    const { queryAllByRole } = renderHarness(jest.fn());

    expect(queryAllByRole("listitem")).toHaveLength(0);
  });

  it("orders results by match quality", async () => {
    const { getByLabelText, findAllByRole } = renderHarness(jest.fn());

    await userEvent.type(getByLabelText("search"), "dialogs");

    const rendered: Array<HTMLElement> = await findAllByRole("listitem");

    expect(rendered[0]).toHaveTextContent("dialogs.xml");
  });

  it("moves the active result with the arrow keys and accepts it with enter", async () => {
    const onSelect = jest.fn();
    const { getByLabelText, getByTestId, findAllByRole } = renderHarness(onSelect);

    const input: HTMLElement = getByLabelText("search");

    await userEvent.type(input, "dialogs");
    await findAllByRole("listitem");

    expect(getByTestId("active")).toHaveTextContent("0");

    await userEvent.keyboard("{ArrowDown}");
    expect(getByTestId("active")).toHaveTextContent("1");

    await userEvent.keyboard("{Enter}");

    // Focus never leaves the field, which is the whole point of driving the list from the input.
    expect(input).toHaveFocus();
    expect(onSelect).toHaveBeenCalledTimes(1);
  });

  it("wraps around at both ends of the list", async () => {
    const { getByLabelText, getByTestId, findAllByRole } = renderHarness(jest.fn());

    await userEvent.type(getByLabelText("search"), "dialogs");
    await findAllByRole("listitem");

    await userEvent.keyboard("{ArrowUp}");
    expect(getByTestId("active")).toHaveTextContent("2");

    await userEvent.keyboard("{ArrowDown}");
    expect(getByTestId("active")).toHaveTextContent("0");
  });

  it("reports the true total while returning only the limit", async () => {
    const { getByLabelText, getByTestId, findAllByRole } = renderHarness(jest.fn(), 1);

    await userEvent.type(getByLabelText("search"), "dialogs");

    expect(await findAllByRole("listitem")).toHaveLength(1);
    expect(getByTestId("total")).toHaveTextContent("3");
  });

  it("sends the active index back to the top when the query changes", async () => {
    const { getByLabelText, getByTestId, findAllByRole } = renderHarness(jest.fn());

    await userEvent.type(getByLabelText("search"), "dialogs");
    await findAllByRole("listitem");

    await userEvent.keyboard("{ArrowDown}");
    expect(getByTestId("active")).toHaveTextContent("1");

    // A new query means a new best match; keeping the old offset points at something unrelated.
    await userEvent.type(getByLabelText("search"), "_");
    expect(getByTestId("active")).toHaveTextContent("0");
  });
});
