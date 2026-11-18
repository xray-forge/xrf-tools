import { afterEach, describe, expect, it, jest } from "@jest/globals";
import { act, fireEvent, RenderResult } from "@testing-library/react";

import { renderWithTheme } from "#/testing/render";

import { FileToast } from "./FileToast";

const FILE: string = "C:\\appdata\\level-viewer\\screenshots\\zaton-20261104-230005.png";

function renderToast(onOpen: () => void = jest.fn(), onDelete: () => void = jest.fn()): RenderResult {
  return renderWithTheme(
    <FileToast
      file={FILE}
      title={"Screenshot saved"}
      name={"zaton-20261104-230005.png"}
      details={"1920 × 1080 · 2.1 MB"}
      preview={<img alt={""} />}
      opening={{ label: "Open the screenshot", onOpen }}
      onReveal={jest.fn()}
      onCopyPath={jest.fn()}
      onDelete={onDelete}
    />
  );
}

describe("FileToast", () => {
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

  // A keyboard user tabbing to an action would otherwise have the toast taken from under the focus.
  it("stays while focus is on one of its actions, and disarms a delete as focus leaves", () => {
    jest.useFakeTimers();

    const onDelete = jest.fn();
    const { getByRole, queryByRole } = renderToast(jest.fn(), onDelete);
    const remove: HTMLElement = getByRole("button", { name: "Delete" });

    fireEvent.focus(remove);
    fireEvent.click(remove);
    act(() => void jest.advanceTimersByTime(10_000));

    expect(queryByRole("status")).not.toBeNull();

    fireEvent.blur(getByRole("button", { name: "Delete for good" }), { relatedTarget: null });

    expect(getByRole("button", { name: "Delete" })).not.toBeNull();

    act(() => void jest.advanceTimersByTime(10_000));

    expect(queryByRole("status")).toBeNull();
    expect(onDelete).not.toHaveBeenCalled();
  });

  it("opens what its preview shows and goes as it does", () => {
    const onOpen = jest.fn();
    const { getByRole, queryByRole } = renderToast(onOpen);

    fireEvent.click(getByRole("button", { name: "Open the screenshot" }));

    expect(onOpen).toHaveBeenCalledTimes(1);
    expect(queryByRole("status")).toBeNull();
  });

  it("deletes only on a second click, and disarms once the pointer leaves", () => {
    const onDelete = jest.fn();
    const { getByRole } = renderToast(jest.fn(), onDelete);

    fireEvent.click(getByRole("button", { name: "Delete" }));

    expect(onDelete).not.toHaveBeenCalled();

    fireEvent.mouseLeave(getByRole("status"));
    fireEvent.click(getByRole("button", { name: "Delete" }));
    fireEvent.click(getByRole("button", { name: "Delete for good" }));

    expect(onDelete).toHaveBeenCalledTimes(1);
  });
});
