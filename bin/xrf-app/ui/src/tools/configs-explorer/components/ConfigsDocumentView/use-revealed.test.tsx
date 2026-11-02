import { describe, expect, it } from "@jest/globals";
import { act } from "@testing-library/react";
import { Container } from "@wirestate/core";
import { Nullable } from "@xrf/types";
import { ReactElement } from "react";

import { mockContainer } from "@/fixtures/utils/container";
import { renderWithProviders } from "@/fixtures/utils/render";
import { useRevealed } from "@/tools/configs-explorer/components/ConfigsDocumentView/use-revealed";
import { TConfigsReveal } from "@/tools/configs-explorer/lib/reveal";
import { ConfigsDocumentService } from "@/tools/configs-explorer/services/configs-document";
import { ConfigsFindingsService } from "@/tools/configs-explorer/services/configs-findings";
import { ConfigsProjectService } from "@/tools/configs-explorer/services/configs-project";
import { ConfigsResolvedService } from "@/tools/configs-explorer/services/configs-resolved";
import { ConfigsSchemeService } from "@/tools/configs-explorer/services/configs-scheme";

/** The section a request names, since these cases are about section reveals. */
function revealedName(reveal: Nullable<TConfigsReveal>): Nullable<string> {
  return reveal?.kind === "section" ? reveal.section : null;
}

interface IRevealProbeProps {
  /** Every value the hook handed back, in order, since a reveal is spent by the render that acts on it. */
  seen: Array<Nullable<string>>;
}

/**
 * A component rather than a bare `renderHook`: components are what the build wraps in `observer()`, and only inside
 * one does reading an observable subscribe to it.
 */
function RevealProbe({ seen }: IRevealProbeProps): ReactElement {
  seen.push(revealedName(useRevealed()));

  return <div />;
}

describe("useRevealed", () => {
  it("hands a request over once and spends it, so the same section can be asked for again", () => {
    const container: Container = mockContainer([
      ConfigsProjectService,
      ConfigsDocumentService,
      ConfigsFindingsService,
      ConfigsResolvedService,
      ConfigsSchemeService,
    ]);
    const documentService: ConfigsDocumentService = container.get(ConfigsDocumentService);
    const seen: Array<Nullable<string>> = [];

    renderWithProviders(<RevealProbe seen={seen} />, { container });

    act(() => documentService.revealSection("wpn_ak74"));

    expect(seen).toContain("wpn_ak74");
    // Spent: what a panel asked for is a request, not a state the document stays in.
    expect(documentService.revealed).toBeNull();
    expect(seen[seen.length - 1]).toBeNull();

    // The second click on a row that is already revealed. Without the clear above this would change no prop, and the
    // listing would have nothing to scroll to.
    act(() => documentService.revealSection("wpn_ak74"));

    expect(seen.filter((section: Nullable<string>) => section === "wpn_ak74")).toHaveLength(2);
  });
});
