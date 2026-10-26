import { describe, expect, it } from "@jest/globals";
import { act, render } from "@testing-library/react";
import { makeAutoObservable, runInAction } from "@wirestate/mobx";
import { ReactElement } from "react";

function ObserverProbe<T extends { value: string }>({ state }: { state: T }): ReactElement {
  return <span>{state.value}</span>;
}

describe("component observer transform", () => {
  it("makes a generic component react to observable changes without an explicit wrapper", () => {
    const state = makeAutoObservable({ value: "before" });
    const { getByText, queryByText } = render(<ObserverProbe state={state} />);

    expect(getByText("before")).toBeInTheDocument();

    act(() => {
      runInAction(() => {
        state.value = "after";
      });
    });

    expect(getByText("after")).toBeInTheDocument();
    expect(queryByText("before")).not.toBeInTheDocument();
  });
});
