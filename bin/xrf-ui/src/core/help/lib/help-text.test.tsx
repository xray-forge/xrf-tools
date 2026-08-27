import { describe, expect, it } from "@jest/globals";
import { render } from "@testing-library/react";

import { renderHelpText } from "@/core/help/lib/help-text";

describe("renderHelpText", () => {
  it("renders backticked spans as code without losing the text around them", () => {
    const { container } = render(<span>{renderHelpText("hide the `wpn_scope` bone")}</span>);

    const code = container.querySelector("code");

    expect(code).not.toBeNull();
    expect(code).toHaveTextContent("wpn_scope");
    expect(container).toHaveTextContent("hide the wpn_scope bone");
  });

  it("returns plain text as it came", () => {
    const { container } = render(<span>{renderHelpText("nothing marked up")}</span>);

    expect(container.querySelector("code")).toBeNull();
    expect(container).toHaveTextContent("nothing marked up");
  });

  it("renders every span of a multi-span string", () => {
    const { container } = render(<span>{renderHelpText("`a` then `b`")}</span>);

    expect(container.querySelectorAll("code")).toHaveLength(2);
  });
});
