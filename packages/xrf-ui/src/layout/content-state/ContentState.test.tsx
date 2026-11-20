import { describe, expect, it, jest } from "@jest/globals";
import { RenderResult } from "@testing-library/react";
import { userEvent } from "@testing-library/user-event";
import { Nullable } from "@xrf/types";

import { renderWithTheme } from "#/testing/render";

import { ContentState, IContentStateProps } from "./ContentState";

type TStateProps = Partial<IContentStateProps<string>> & { value: Nullable<string> };

function renderState(props: TStateProps): RenderResult {
  return renderWithTheme(
    <ContentState<string>
      data-testid={"chunk"}
      loadingLabel={"Reading chunk…"}
      errorTitle={"Could not read this chunk"}
      emptyTitle={"Nothing to show"}
      emptyDescription={"Open a spawn file to read its chunks."}
      {...props}
    >
      {(value: string) => <p data-testid={"content"}>{value}</p>}
    </ContentState>
  );
}

describe("ContentState", () => {
  it("draws the value it was given", () => {
    const { getByTestId, queryByTestId } = renderState({ value: "header" });

    expect(getByTestId("content")).toHaveTextContent("header");
    expect(queryByTestId("chunk")).not.toBeInTheDocument();
  });

  it("names a pending read while there is nothing to show", () => {
    const { getByRole, getByTestId } = renderState({ value: null, isLoading: true });

    expect(getByTestId("chunk")).toBeInTheDocument();
    expect(getByRole("status")).toHaveTextContent("Reading chunk…");
  });

  it("keeps a held value on screen through a reload", () => {
    const { getByTestId, queryByRole } = renderState({ value: "header", isLoading: true });

    expect(getByTestId("content")).toHaveTextContent("header");
    expect(queryByRole("status")).not.toBeInTheDocument();
  });

  it("announces a failure over a held value and retries through its owner", async () => {
    const onRetry = jest.fn();
    const { getByRole, queryByTestId } = renderState({ value: "header", error: new Error("Truncated"), onRetry });

    expect(getByRole("alert")).toHaveTextContent("Could not read this chunk");
    expect(getByRole("alert")).toHaveTextContent("Truncated");
    expect(queryByTestId("content")).not.toBeInTheDocument();

    await userEvent.click(getByRole("button", { name: "Retry" }));

    expect(onRetry).toHaveBeenCalledTimes(1);
  });

  it("says a finished read found nothing", () => {
    const { getByTestId } = renderState({ value: null });

    expect(getByTestId("chunk")).toHaveTextContent("Nothing to show");
    expect(getByTestId("chunk")).toHaveTextContent("Open a spawn file to read its chunks.");
  });
});
