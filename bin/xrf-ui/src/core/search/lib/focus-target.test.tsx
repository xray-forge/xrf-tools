import { describe, expect, it } from "@jest/globals";
import { act } from "@testing-library/react";
import { CommandBus, Container } from "@wirestate/core";
import { ReactElement, useRef } from "react";

import { FOCUS_SEARCH_FIELD_MESSAGE, useSearchFocusTarget } from "@/core/search/lib";
import { mockContainer } from "@/fixtures/utils/container";
import { renderWithProviders } from "@/fixtures/utils/render";
import { Nullable } from "@/lib/types/general";

function SearchField(): ReactElement {
  const inputRef = useRef<Nullable<HTMLInputElement>>(null);

  useSearchFocusTarget(inputRef);

  return <input ref={inputRef} aria-label={"field"} defaultValue={"typed"} />;
}

describe("useSearchFocusTarget", () => {
  it("takes the caret and selects what is there, so the next keystroke replaces it", () => {
    const container: Container = mockContainer();
    const { getByLabelText } = renderWithProviders(<SearchField />, { container });

    act(() => void container.get(CommandBus).execute(FOCUS_SEARCH_FIELD_MESSAGE, undefined, { optional: true }));

    const field: HTMLInputElement = getByLabelText("field") as HTMLInputElement;

    expect(field).toHaveFocus();
    expect(field.selectionStart).toBe(0);
    expect(field.selectionEnd).toBe("typed".length);
  });

  it("is the only state anything needs: the field answers while it is mounted and not after", () => {
    const container: Container = mockContainer();
    const { unmount } = renderWithProviders(<SearchField />, { container });

    expect(container.get(CommandBus).hasHandler(FOCUS_SEARCH_FIELD_MESSAGE)).toBe(true);

    unmount();

    expect(container.get(CommandBus).hasHandler(FOCUS_SEARCH_FIELD_MESSAGE)).toBe(false);
  });
});
