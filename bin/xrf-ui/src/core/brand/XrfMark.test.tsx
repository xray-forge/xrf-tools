import { describe, expect, it } from "@jest/globals";
import { render } from "@testing-library/react";

import { XrfMark } from "@/core/brand/XrfMark";
import { EBuildKind } from "@/core/ipc/types/xrf-build-info";
import { setMockBuildKind } from "@/fixtures/mocks/build-kind.mocks";

describe("XrfMark", () => {
  it("colours its core by the kind of build that is running", () => {
    const cores: Record<EBuildKind, string> = {
      [EBuildKind.OPTIMIZED]: "#FFA200",
      [EBuildKind.DEVELOPMENT]: "#8B9AFF",
      [EBuildKind.LOCAL]: "#00C369",
    };

    for (const [kind, core] of Object.entries(cores)) {
      setMockBuildKind(kind);

      const { container, unmount } = render(<XrfMark />);

      expect(container.querySelector("polygon")).toHaveAttribute("fill", core);
      unmount();
    }
  });
});
