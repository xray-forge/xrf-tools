import { afterEach, beforeEach, describe, expect, it, jest } from "@jest/globals";
import { act, render } from "@testing-library/react";
import { userEvent } from "@testing-library/user-event";
import { ReactElement } from "react";

import { COPIED_FOR, useCopiedText } from "@/lib/react/use-copied-text";

function Component({ onFailure }: { onFailure: (error: unknown) => void }): ReactElement {
  const { isCopied, copy } = useCopiedText(onFailure);

  return <button onClick={() => copy("text")}>{isCopied ? "Copied" : "Copy"}</button>;
}

describe("useCopiedText", () => {
  const writeText = jest.fn<(text: string) => Promise<void>>();

  beforeEach(() => {
    Object.defineProperty(navigator, "clipboard", { configurable: true, value: { writeText } });
  });

  afterEach(() => {
    jest.useRealTimers();
    writeText.mockReset();
  });

  it("says a copy is done for a moment, then no longer", async () => {
    writeText.mockResolvedValue();
    jest.useFakeTimers({ advanceTimers: true });

    const { getByRole } = render(<Component onFailure={() => {}} />);

    await userEvent.click(getByRole("button"));

    expect(writeText).toHaveBeenCalledWith("text");
    expect(getByRole("button").textContent).toBe("Copied");

    act(() => jest.advanceTimersByTime(COPIED_FOR));

    expect(getByRole("button").textContent).toBe("Copy");
  });

  it("tells its owner why a copy the clipboard refused failed, and says nothing was copied", async () => {
    const onFailure = jest.fn();

    writeText.mockRejectedValue(new Error("denied"));

    const { getByRole } = render(<Component onFailure={onFailure} />);

    await userEvent.click(getByRole("button"));

    expect(onFailure).toHaveBeenCalledWith(new Error("denied"));
    expect(getByRole("button").textContent).toBe("Copy");
  });
});
