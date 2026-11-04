import { afterEach, describe, expect, it, jest } from "@jest/globals";
import { act, fireEvent, render } from "@testing-library/react";

import { LevelViewerFileToast } from "@/tools/level-viewer/components/toast/LevelViewerFileToast";

const FILE: string = "C:\\appdata\\level-viewer\\screenshots\\zaton-20261104-230005.png";

function renderToast(onOpen: () => void = jest.fn()): ReturnType<typeof render> {
  return render(
    <LevelViewerFileToast
      file={FILE}
      title={"Screenshot saved"}
      name={"zaton-20261104-230005.png"}
      details={"1920 × 1080 · 2.1 MB"}
      preview={<img alt={""} />}
      opening={{ label: "Open the screenshot", onOpen }}
      onReveal={jest.fn()}
      onCopyPath={jest.fn()}
      onDelete={jest.fn()}
    />
  );
}

describe("LevelViewerFileToast", () => {
  afterEach(() => {
    jest.useRealTimers();
  });

  it("goes a while after it shows, unless the pointer rests on it", () => {
    jest.useFakeTimers();

    const { getByRole, queryByRole } = renderToast();

    fireEvent.mouseEnter(getByRole("status", { name: "Screenshot saved" }));
    act(() => void jest.advanceTimersByTime(10_000));

    expect(queryByRole("status")).not.toBeNull();

    fireEvent.mouseLeave(getByRole("status"));
    act(() => void jest.advanceTimersByTime(10_000));

    expect(queryByRole("status")).toBeNull();
  });

  it("opens what its preview shows and goes as it does", () => {
    const onOpen = jest.fn();
    const { getByRole, queryByRole } = renderToast(onOpen);

    fireEvent.click(getByRole("button", { name: "Open the screenshot" }));

    expect(onOpen).toHaveBeenCalledTimes(1);
    expect(queryByRole("status")).toBeNull();
  });
});
