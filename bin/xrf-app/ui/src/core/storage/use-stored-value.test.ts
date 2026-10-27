import { beforeEach, describe, expect, it } from "@jest/globals";
import { act, renderHook } from "@testing-library/react";
import { Nullable } from "@xrf/types";

import { clearLocalStorage, getLocalStorageValue, setLocalStorageValue } from "@/core/storage/read-write";
import { IStoredValueCodec, textCodec } from "@/core/storage/stored-value-codec";
import { useStoredValue } from "@/core/storage/use-stored-value";

const KEY: string = "xrf.preference.fixture";

const TEXT_CODEC: IStoredValueCodec<string> = textCodec((stored: Nullable<string>) => stored ?? "none");

describe("useStoredValue", () => {
  beforeEach(() => {
    clearLocalStorage();
  });

  it("holds and writes each value set", () => {
    const { result } = renderHook(() => useStoredValue(KEY, TEXT_CODEC));

    act(() => result.current[1]("b"));

    expect(result.current[0]).toBe("b");
    expect(getLocalStorageValue(KEY)).toBe("b");
  });

  it("reads the new key's value on the render the key changes", () => {
    setLocalStorageValue(`${KEY}.a`, "first");
    setLocalStorageValue(`${KEY}.b`, "second");

    const { result, rerender } = renderHook(({ key }) => useStoredValue(key, TEXT_CODEC), {
      initialProps: { key: `${KEY}.a` },
    });

    expect(result.current[0]).toBe("first");

    rerender({ key: `${KEY}.b` });

    expect(result.current[0]).toBe("second");
  });
});
