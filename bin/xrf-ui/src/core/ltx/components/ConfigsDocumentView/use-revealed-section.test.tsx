import { describe, expect, it } from "@jest/globals";
import { act } from "@testing-library/react";
import { Container } from "@wirestate/core";
import { ReactElement } from "react";

import { useRevealedSection } from "@/core/ltx/components/ConfigsDocumentView/use-revealed-section";
import { ConfigsDocumentService } from "@/core/ltx/services/document";
import { ConfigsProjectService } from "@/core/ltx/services/project";
import { mockContainer } from "@/fixtures/utils/container";
import { renderWithProviders } from "@/fixtures/utils/render";
import { Nullable } from "@/lib/types/general";

interface IRevealProbeProps {
  /** Every value the hook handed back, in order, since a reveal is spent by the render that acts on it. */
  seen: Array<Nullable<string>>;
}

/**
 * A component rather than a bare `renderHook`: components are what the build wraps in `observer()`, and only inside
 * one does reading an observable subscribe to it.
 */
function RevealProbe({ seen }: IRevealProbeProps): ReactElement {
  seen.push(useRevealedSection());

  return <div />;
}

describe("useRevealedSection", () => {
  it("hands a request over once and spends it, so the same section can be asked for again", () => {
    const container: Container = mockContainer([ConfigsProjectService, ConfigsDocumentService]);
    const documentService: ConfigsDocumentService = container.get(ConfigsDocumentService);
    const seen: Array<Nullable<string>> = [];

    renderWithProviders(<RevealProbe seen={seen} />, { container });

    act(() => documentService.revealSection("wpn_ak74"));

    expect(seen).toContain("wpn_ak74");
    // Spent: what a panel asked for is a request, not a state the document stays in.
    expect(documentService.revealedSection).toBeNull();
    expect(seen[seen.length - 1]).toBeNull();

    // The second click on a row that is already revealed. Without the clear above this would change no prop, and the
    // listing would have nothing to scroll to.
    act(() => documentService.revealSection("wpn_ak74"));

    expect(seen.filter((section: Nullable<string>) => section === "wpn_ak74")).toHaveLength(2);
  });
});
