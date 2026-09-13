import { describe, expect, it } from "@jest/globals";

import { SyntaxContent } from "@/core/syntax/components/SyntaxContent";
import { ESyntaxLanguage } from "@/core/syntax/lib";
import { renderWithProviders } from "@/fixtures/utils/render";

const LTX: string = "; comment\n[wpn_ak74]:wpn_base\nammo_mag_size = 30\n";

describe("SyntaxContent", () => {
  it("renders the source verbatim, whatever it colours", () => {
    // A preview that quietly drops or reorders a character is worse than one with no colour at all.
    const { container } = renderWithProviders(<SyntaxContent content={LTX} language={ESyntaxLanguage.LTX} />);

    expect(container.textContent).toBe(LTX);
  });

  it("wraps only what it colours, leaving the rest as bare text", () => {
    const { container } = renderWithProviders(<SyntaxContent content={LTX} language={ESyntaxLanguage.LTX} />);
    const colored: Array<string> = Array.from(container.querySelectorAll<HTMLSpanElement>("[data-syntax-token]")).map(
      (span: HTMLSpanElement) => span.textContent as string
    );

    expect(colored).toContain("; comment");
    expect(colored).toContain("[wpn_ak74]");
    expect(colored).toContain("ammo_mag_size");
    // Whitespace and plain runs stay unwrapped: at one node per run, a large file cannot afford them.
    expect(colored).not.toContain("\n");
  });

  it("emits no token elements for a language it does not colour", () => {
    const { container } = renderWithProviders(<SyntaxContent content={LTX} language={ESyntaxLanguage.PLAIN} />);

    expect(container.querySelectorAll("[data-syntax-token]")).toHaveLength(0);
    expect(container.textContent).toBe(LTX);
  });
});
