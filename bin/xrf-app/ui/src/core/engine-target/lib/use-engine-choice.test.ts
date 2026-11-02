import { afterEach, describe, expect, it } from "@jest/globals";
import { renderHook } from "@testing-library/react";
import { EAssetsCommand } from "@xrf/ipc/commands/assets";
import { mockInvoke, resetMockInvoke } from "@xrf/ipc/testing";

import { useEngineChoice } from "@/core/engine-target/lib/use-engine-choice";

describe("useEngineChoice", () => {
  afterEach(() => {
    resetMockInvoke();
    mockInvoke.mockClear();
  });

  // An empty field names no root, as none picked does: nothing is detected, and nothing waits on it.
  it("detects nothing for an empty root, and says it is not detecting", () => {
    for (const root of [null, ""]) {
      const { result } = renderHook(() => useEngineChoice(root));

      expect(result.current.isDetecting).toBe(false);
      expect(result.current.detection).toBeNull();
    }

    expect(mockInvoke.mock.calls.filter(([name]) => name === EAssetsCommand.DETECT_ENGINE)).toHaveLength(0);
  });
});
