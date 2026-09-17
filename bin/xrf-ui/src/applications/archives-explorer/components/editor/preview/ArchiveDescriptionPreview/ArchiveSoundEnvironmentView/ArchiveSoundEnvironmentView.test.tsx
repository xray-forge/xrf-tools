import { describe, expect, it } from "@jest/globals";
import { RenderResult } from "@testing-library/react";

import { ArchiveSoundEnvironmentDescription } from "@/core/ipc/types/xrf-app";
import { mockArchiveSoundEnvironmentDescription } from "@/fixtures/mocks/archive.mocks";
import { renderWithProviders } from "@/fixtures/utils/render";

import { ArchiveSoundEnvironmentView } from "./ArchiveSoundEnvironmentView";

function renderView(
  description: ArchiveSoundEnvironmentDescription = mockArchiveSoundEnvironmentDescription()
): RenderResult {
  return renderWithProviders(<ArchiveSoundEnvironmentView description={description} />);
}

describe("ArchiveSoundEnvironmentView", () => {
  it("lists every preset in file order, which is how a level addresses one", () => {
    const { getByText } = renderView();

    expect(getByText("Reverb presets (2)")).toBeTruthy();
    expect(getByText(/how a level addresses one/)).toBeTruthy();
  });

  it("leads with the two figures a listener hears first", () => {
    const { getByText } = renderView();

    expect(getByText("2.91 s decay · 14.6 m")).toBeTruthy();
  });

  it("names the EAX preset a record stands for where it declares one", () => {
    const { getByText } = renderView();

    expect(getByText("room -1000 · high -602 · EAX preset 26")).toBeTruthy();
  });

  it("leaves an older preset unqualified rather than inventing a preset number for it", () => {
    // Only version 4 and above carries one; the engine reads nothing there for a version 3 record.
    const { getByText } = renderView();

    expect(getByText("room -1000 · high -1000")).toBeTruthy();
  });
});
