import { describe, expect, it } from "@jest/globals";
import { RenderResult } from "@testing-library/react";

import { ArchivesService } from "@/applications/archives-explorer/services/archives";
import { ArchiveDescribeScope, ArchivePpeDescription, EArchiveDescribeScope } from "@/core/ipc/types/xrf-app";
import {
  mockArchiveAnimationChannel,
  mockArchivePpeColor,
  mockArchivePpeDescription,
} from "@/fixtures/mocks/archive.mocks";
import { mockInjectedService } from "@/fixtures/utils/container";
import { renderWithProviders } from "@/fixtures/utils/render";

import { ArchivePpeDescriptionView } from "./ArchivePpeDescriptionView";

const SCOPE: ArchiveDescribeScope = { kind: EArchiveDescribeScope.VOLUMES, volumes: 3 };

function renderView(description: ArchivePpeDescription = mockArchivePpeDescription()): RenderResult {
  const { container } = mockInjectedService(ArchivesService);

  return renderWithProviders(<ArchivePpeDescriptionView description={description} scope={SCOPE} />, { container });
}

describe("ArchivePpeDescriptionView", () => {
  it("leads with how long the effect runs, which is its longest parameter", () => {
    const { getByText } = renderView();

    expect(getByText("5.00 s")).toBeTruthy();
    expect(getByText(/longest parameter decides it, not where its last key sits/)).toBeTruthy();
  });

  it("says how much of the effect is actually keyed, because most of it is not", () => {
    // Base colour and gray value and the grading influence: 3 of the 11 parameters an effect always stores.
    const { getByText } = renderView();

    expect(getByText("Across 3 of the 11 parameters; the rest are stored empty")).toBeTruthy();
  });

  it("says plainly when an effect changes nothing at all", () => {
    const { getByText } = renderView(
      mockArchivePpeDescription({
        colors: [mockArchivePpeColor("base color", { keys: 0 })],
        values: [mockArchiveAnimationChannel("blur", { keys: 0 })],
        colorMap: null,
      })
    );

    expect(getByText("Across none of the 2 parameters, so the effect changes nothing")).toBeTruthy();
  });

  it("offers the gradient the grading samples as somewhere to go", () => {
    const { getByText } = renderView();

    expect(getByText("grad\\grad_psi").closest("button")).not.toBeNull();
  });

  it("says an effect carrying the grading parameter but no gradient grades nothing", () => {
    const { getByText } = renderView(
      mockArchivePpeDescription({
        colorMap: {
          texture: null,
          influence: mockArchiveAnimationChannel("colour map influence", { keys: 0 }),
          isUsed: false,
        },
      })
    );

    expect(getByText(/names no gradient, so it grades nothing/)).toBeTruthy();
  });

  it("names the version that predates colour grading", () => {
    const { getByText } = renderView(mockArchivePpeDescription({ version: 1, colorMap: null }));

    expect(getByText("Below version 2, which is where colour grading was added")).toBeTruthy();
  });

  it("gives each colour its own section over the three channels the engine assembles it from", () => {
    const { getByText, getAllByText } = renderView();

    expect(getByText("base color")).toBeTruthy();
    expect(getByText("add color")).toBeTruthy();
    expect(getByText("gray color")).toBeTruthy();
    expect(getAllByText("red")).toHaveLength(3);
  });

  it("carries the colour base the engine stores and never reads", () => {
    const { getByText, getAllByText } = renderView();

    expect(getAllByText(/base 0\.500, which the engine stores and never reads/)).toHaveLength(3);
    expect(getByText(/4 keys over 3\.00 s · base/)).toBeTruthy();
    expect(getAllByText(/No keys on any channel · base/)).toHaveLength(2);
  });

  it("lists every scalar parameter, including the ones nothing keys", () => {
    const { getByText, getAllByText } = renderView();

    expect(getByText("gray value")).toBeTruthy();
    expect(getByText("noise fps")).toBeTruthy();
    // Six empty scalars plus the blue channel of each of the three colours.
    expect(getAllByText("No keys")).toHaveLength(9);
  });

  it("lists the grading influence among the scalars, because that is what it is", () => {
    const { getByText } = renderView();

    expect(getByText("colour map influence")).toBeTruthy();
    expect(getByText("1 key at 0.00 s")).toBeTruthy();
  });
});
