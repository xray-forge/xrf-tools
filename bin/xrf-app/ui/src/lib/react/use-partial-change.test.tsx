import { describe, expect, it, jest } from "@jest/globals";
import { renderHook } from "@testing-library/react";

import { usePartialChange } from "@/lib/react/use-partial-change";

describe("usePartialChange", () => {
  it("tells the whole value with the part set over it", () => {
    const onChange = jest.fn();
    const { result } = renderHook(() => usePartialChange({ far: 350, near: 10 }, onChange));

    result.current({ far: 500 });

    expect(onChange).toHaveBeenCalledWith({ far: 500, near: 10 });
  });
});
