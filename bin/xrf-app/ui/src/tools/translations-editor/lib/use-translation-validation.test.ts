import { describe, expect, it, jest } from "@jest/globals";
import { act, renderHook, RenderHookResult } from "@testing-library/react";
import { Nullable } from "@xrf/types";

import {
  ITranslationValidation,
  IUseTranslationValidationOptions,
  TValidateText,
  useTranslationValidation,
} from "./use-translation-validation";

interface IPendingAnswer {
  text: string;
  answer: (error: Nullable<string>) => Promise<void>;
}

function renderValidation(initial: Omit<IUseTranslationValidationOptions, "validateText">): {
  hook: RenderHookResult<ITranslationValidation, Omit<IUseTranslationValidationOptions, "validateText">>;
  asked: Array<IPendingAnswer>;
} {
  const asked: Array<IPendingAnswer> = [];
  const validateText: TValidateText = jest.fn(
    (_language: string, text: string) =>
      new Promise<Nullable<string>>((resolve) => {
        asked.push({
          text,
          answer: async (error: Nullable<string>) => {
            await act(async () => resolve(error));
          },
        });
      })
  );
  const hook = renderHook(
    (props: Omit<IUseTranslationValidationOptions, "validateText">) =>
      useTranslationValidation({ ...props, validateText }),
    { initialProps: initial }
  );

  return { hook, asked };
}

describe("useTranslationValidation", () => {
  it("reports what the backend found wrong with a committed value, and clears it once fixed", async () => {
    const { hook, asked } = renderValidation({ file: "text/eng/st_items.xml", language: "eng" });

    act(() => hook.result.current.validate("st_bread", "хлеб"));
    await asked[0].answer("х");

    expect(hook.result.current.getErrorOf("st_bread")).toBe("х");

    act(() => hook.result.current.validate("st_bread", "bread"));
    await asked[1].answer(null);

    expect(hook.result.current.getErrorOf("st_bread")).toBeNull();
  });

  // Typing on outruns the backend: an answer for a value already replaced must not paint the cell.
  it("discards an answer for a value edited again since", async () => {
    const { hook, asked } = renderValidation({ file: "text/eng/st_items.xml", language: "eng" });

    act(() => hook.result.current.validate("st_bread", "хлеб"));
    act(() => hook.result.current.validate("st_bread", "bread"));
    await asked[1].answer(null);
    await asked[0].answer("х");

    expect(hook.result.current.getErrorOf("st_bread")).toBeNull();
  });

  // The same id names another cell in another file or language, whose error is not this one's.
  it("keeps a cell's error to its file and language", async () => {
    const { hook, asked } = renderValidation({ file: "text/eng/st_items.xml", language: "eng" });

    act(() => hook.result.current.validate("st_bread", "хлеб"));
    await asked[0].answer("х");

    hook.rerender({ file: "text/eng/st_items.xml", language: "rus" });
    expect(hook.result.current.getErrorOf("st_bread")).toBeNull();

    hook.rerender({ file: "text/eng/st_dialogs.xml", language: "eng" });
    expect(hook.result.current.getErrorOf("st_bread")).toBeNull();

    hook.rerender({ file: "text/eng/st_items.xml", language: "eng" });
    expect(hook.result.current.getErrorOf("st_bread")).toBe("х");
  });

  it("validates nothing while no file is open", () => {
    const { hook, asked } = renderValidation({ file: null, language: "eng" });

    act(() => hook.result.current.validate("st_bread", "хлеб"));

    expect(asked).toHaveLength(0);
    expect(hook.result.current.getErrorOf("st_bread")).toBeNull();
  });
});
