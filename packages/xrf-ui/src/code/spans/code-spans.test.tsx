import { describe, expect, it } from "@jest/globals";

import { renderWithTheme } from "#/testing/render";

import { renderCodeSpans } from "./code-spans";

describe("renderCodeSpans", () => {
  it("renders backticked spans as code without losing the text around them", () => {
    const { container } = renderWithTheme(<span>{renderCodeSpans("hide the `wpn_scope` bone")}</span>);

    const code = container.querySelector("code");

    expect(code).not.toBeNull();
    expect(code).toHaveTextContent("wpn_scope");
    expect(container).toHaveTextContent("hide the wpn_scope bone");
  });

  it("returns plain text as it came", () => {
    const { container } = renderWithTheme(<span>{renderCodeSpans("nothing marked up")}</span>);

    expect(container.querySelector("code")).toBeNull();
    expect(container).toHaveTextContent("nothing marked up");
  });

  it("renders every span of a multi-span string", () => {
    const { container } = renderWithTheme(<span>{renderCodeSpans("`a` then `b`")}</span>);

    expect(container.querySelectorAll("code")).toHaveLength(2);
  });
});
