import { beforeEach, describe, expect, it } from "@jest/globals";
import { act, renderHook } from "@testing-library/react";

import { EApplicationId } from "@/core/routing/application";
import { clearLocalStorage, getLocalStorageValue, setLocalStorageValue } from "@/core/storage";
import { useRememberedValue } from "@/core/ui/form/use-remembered-value";

describe("useRememberedValue", () => {
  const STORAGE_KEY: string = "xrf.form.translations-parser.language";

  function renderField() {
    return renderHook(() =>
      useRememberedValue({
        application: EApplicationId.TRANSLATIONS_PARSER,
        id: "language",
        fallback: "eng",
        allowed: ["eng", "ukr"],
      })
    );
  }

  beforeEach(() => {
    clearLocalStorage();
  });

  it("starts on the fallback when nothing was remembered", () => {
    expect(renderField().result.current[0]).toBe("eng");
  });

  it("restores what was remembered", () => {
    setLocalStorageValue(STORAGE_KEY, "ukr");

    expect(renderField().result.current[0]).toBe("ukr");
  });

  it("records what it is set to", () => {
    const { result } = renderField();

    act(() => result.current[1]("ukr"));

    expect(result.current[0]).toBe("ukr");
    expect(getLocalStorageValue(STORAGE_KEY)).toBe("ukr");
  });

  // A value the form no longer offers must not come back from a machine that used it before, or the
  // field starts on something its own control cannot display.
  it("ignores a remembered value the field no longer accepts", () => {
    setLocalStorageValue(STORAGE_KEY, "rus");

    expect(renderField().result.current[0]).toBe("eng");
  });
});
