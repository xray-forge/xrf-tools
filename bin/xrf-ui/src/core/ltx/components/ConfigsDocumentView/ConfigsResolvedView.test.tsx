import { describe, expect, it } from "@jest/globals";
import { RenderResult } from "@testing-library/react";
import { userEvent } from "@testing-library/user-event";
import { Container } from "@wirestate/core";

import { LtxResolvedIndex } from "@/core/bindings/types/xrf-ltx-inspect";
import { ConfigsResolvedView } from "@/core/ltx/components/ConfigsDocumentView/ConfigsResolvedView";
import { ConfigsDocumentService } from "@/core/ltx/services/document";
import { ConfigsProjectService } from "@/core/ltx/services/project";
import { ConfigsResolvedService } from "@/core/ltx/services/resolved";
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
function renderResolved(narrowedTo: string | null): RenderResult {
  const container: Container = mockContainer([ConfigsProjectService, ConfigsDocumentService, ConfigsResolvedService]);
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
    const render_: RenderResult = renderResolved(INCLUDER);

    expect(render_.getByText("This config declares no sections")).toBeInTheDocument();
    expect(render_.getByText(new RegExp(INCLUDER.replace(/\\/g, "\\\\")))).toBeInTheDocument();
    expect(render_.queryByTestId("configs-resolved-lines")).not.toBeInTheDocument();

    // The way out is the same one the banner offers, because the reader's next question is what the root does hold.
    await userEvent.click(render_.getByRole("button", { name: "Show all" }));

    expect(render_.getByTestId("configs-resolved-lines")).toBeInTheDocument();
    expect(render_.queryByText("This config declares no sections")).not.toBeInTheDocument();
  });

  it("draws the resolution when it holds sections", () => {
    const render_: RenderResult = renderResolved(null);

    expect(render_.getByTestId("configs-resolved-lines")).toBeInTheDocument();
    expect(render_.getByText("[wpn_base]")).toBeInTheDocument();
  });
});
