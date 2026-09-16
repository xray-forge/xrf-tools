import { describe, expect, it } from "@jest/globals";
import { RenderResult } from "@testing-library/react";

import { ArchiveEfdDescription } from "@/core/ipc/types/xrf-app";
import { mockArchiveEfdDescription } from "@/fixtures/mocks/archive.mocks";
import { renderWithProviders } from "@/fixtures/utils/render";

import { ArchiveEfdView } from "./ArchiveEfdView";

function renderView(description: ArchiveEfdDescription = mockArchiveEfdDescription()): RenderResult {
  return renderWithProviders(<ArchiveEfdView description={description} />);
}

describe("ArchiveEfdView", () => {
  it("says what the number beside the function is, since the number alone names nothing", () => {
    // vfLoadEF writes the loaded function into m_fpaBaseFunctions[m_dwFunctionType], so the value is a slot.
    const { getByText } = renderView();

    expect(getByText("Function 70")).toBeTruthy();
    expect(getByText(/slot it claims in the engine's function table/)).toBeTruthy();
  });

  it("gives the band the trained table answers in", () => {
    const { getByText } = renderView();

    expect(getByText("0.000 to 1.000")).toBeTruthy();
  });

  it("says the weight count is derived rather than stored, which is why the terms are worth reading", () => {
    const { getByText } = renderView();

    expect(getByText("40")).toBeTruthy();
    expect(getByText(/file stores no count of them/)).toBeTruthy();
  });

  it("numbers the inputs as the terms address them, from zero", () => {
    const { getByText } = renderView();

    expect(getByText("Input 0")).toBeTruthy();
    expect(getByText("10 buckets")).toBeTruthy();
    expect(getByText("Read from function 21")).toBeTruthy();
  });

  it("says where a term's weights come from, which is the product of its inputs' ranges", () => {
    const { getByText } = renderView();

    expect(getByText("Inputs 0, 1")).toBeTruthy();
    expect(getByText("40 weights, the product of those inputs' ranges")).toBeTruthy();
  });

  it("admits a term whose product cannot be taken rather than reporting a count it does not have", () => {
    const { getByText } = renderView(mockArchiveEfdDescription({ patterns: [{ variables: [0, 1], weights: null }] }));

    expect(getByText("More weights than the ranges can be multiplied into")).toBeTruthy();
  });

  it("reads a singular bucket as one", () => {
    const { getByText } = renderView(mockArchiveEfdDescription({ variableRanges: [1], variableKinds: [0] }));

    expect(getByText("1 bucket")).toBeTruthy();
  });
});
