import { describe, expect, it } from "@jest/globals";
import { RenderResult } from "@testing-library/react";
import { userEvent } from "@testing-library/user-event";
import { Container } from "@wirestate/core";
import { LtxResolvedIndex } from "@xrf/ipc/types/xrf-ltx-inspect";
import { Nullable } from "@xrf/types";

import { ConfigsResolvedView } from "@/applications/configs-explorer/components/ConfigsDocumentView/ConfigsResolvedView";
import { ConfigsDocumentService } from "@/applications/configs-explorer/services/document";
import { ConfigsFindingsService } from "@/applications/configs-explorer/services/findings";
import { ConfigsProjectService } from "@/applications/configs-explorer/services/project";
import { ConfigsResolvedService } from "@/applications/configs-explorer/services/resolved";
import { ConfigsSchemeService } from "@/applications/configs-explorer/services/scheme";
import { mockContainer } from "@/fixtures/utils/container";
import { renderWithProviders } from "@/fixtures/utils/render";

const ENTRY: string = "configs\\system.ltx";
const INCLUDER: string = "configs\\includes_only.ltx";

/** An index holding one section, declared by a config that is not the one the view narrows to. */
const INDEX: LtxResolvedIndex = {
  dialect: "ltx",
  diagnostics: [],
  entry: ENTRY,
  sections: [{ fieldCount: 1, name: "wpn_base", origin: "configs\\items\\w_base.ltx", parents: [] }],
};

/** The view over a resolution that is already indexed, narrowed to whichever config a case is about. */
function renderResolved(narrowedTo: Nullable<string>): RenderResult {
  const container: Container = mockContainer([
    ConfigsProjectService,
    ConfigsDocumentService,
    ConfigsResolvedService,
    ConfigsFindingsService,
    ConfigsSchemeService,
  ]);
  const service: ConfigsResolvedService = container.get(ConfigsResolvedService);

  service.entry = ENTRY;
  service.index = service.index.asReady(INDEX);
  service.narrowTo(narrowedTo);

  return renderWithProviders(<ConfigsResolvedView />, { container });
}

describe("ConfigsResolvedView", () => {
  it("says what an empty resolution means rather than drawing a blank document", async () => {
    // A config that only includes others resolves to nothing of its own, and so does one whose sections a patch
    // deleted. Both are real files to open, and neither is a failure.
    const { getByRole, getByTestId, getByText, queryByTestId, queryByText } = renderResolved(INCLUDER);

    expect(getByText("This config declares no sections")).toBeInTheDocument();
    expect(getByText(new RegExp(INCLUDER.replace(/\\/g, "\\\\")))).toBeInTheDocument();
    expect(queryByTestId("configs-resolved-lines")).not.toBeInTheDocument();

    // The way out is the same one the banner offers, because the reader's next question is what the root does hold.
    await userEvent.click(getByRole("button", { name: "Show all" }));

    expect(getByTestId("configs-resolved-lines")).toBeInTheDocument();
    expect(queryByText("This config declares no sections")).not.toBeInTheDocument();
  });

  it("draws the resolution when it holds sections", () => {
    const { getByTestId, getByText } = renderResolved(null);

    expect(getByTestId("configs-resolved-lines")).toBeInTheDocument();
    expect(getByText("[wpn_base]")).toBeInTheDocument();
  });
});
