import { describe, expect, it } from "@jest/globals";
import { render } from "@testing-library/react";

import { XrfMark, XrfMarkCore } from "./XrfMark";

describe("XrfMark", () => {
  it("colours its core by the build it names", () => {
    const cores: Record<XrfMarkCore, string> = {
      optimized: "#FFA200",
      development: "#8B9AFF",
      local: "#00C369",
    };

    for (const [core, colour] of Object.entries(cores) as Array<[XrfMarkCore, string]>) {
      const { container, unmount } = render(<XrfMark core={core} />);

      expect(container.querySelector("polygon")).toHaveAttribute("fill", colour);
      unmount();
    }
  });
});
