import { describe, expect, it } from "@jest/globals";
import { ReactElement } from "react";

import { renderWithProviders } from "@/fixtures/utils/render";

import { TChunkTabChange, useChunkTab } from "./use-chunk-tab";

const BASE_PATH: string = "/spawn-editor/graph";
const TABS: Array<string> = ["header", "levels", "vertices"];

function TabRenderer(): ReactElement {
  const [activeTab]: [string, TChunkTabChange] = useChunkTab(BASE_PATH, TABS, "header");

  return <span data-testid={"active"}>{activeTab}</span>;
}

describe("useChunkTab", () => {
  it("takes the active sub-table from the route", () => {
    const { getByTestId } = renderWithProviders(<TabRenderer />, { route: `${BASE_PATH}/vertices` });

    expect(getByTestId("active")).toHaveTextContent("vertices");
  });

  it("falls back when the route names no sub-table", () => {
    const { getByTestId } = renderWithProviders(<TabRenderer />, { route: BASE_PATH });

    expect(getByTestId("active")).toHaveTextContent("header");
  });

  it("falls back when the route names one that does not exist", () => {
    const { getByTestId } = renderWithProviders(<TabRenderer />, { route: `${BASE_PATH}/nonsense` });

    expect(getByTestId("active")).toHaveTextContent("header");
  });
});
