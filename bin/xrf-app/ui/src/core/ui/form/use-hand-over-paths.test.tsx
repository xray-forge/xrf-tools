import { beforeEach, describe, expect, it } from "@jest/globals";
import { userEvent } from "@testing-library/user-event";
import { ReactElement } from "react";
import { useLocation } from "react-router-dom";

import { EToolId } from "@/core/routing/tool";
import { useToolBusy } from "@/core/shell/lifecycle";
import { clearLocalStorage, getFieldValueStorageKey, getLocalStorageValue } from "@/core/storage";
import { THandOverPaths, useHandOverPaths } from "@/core/ui/form/use-hand-over-paths";
import { renderWithProviders } from "@/fixtures/utils/render";

function HandOver({ isBusy }: { isBusy: boolean }): ReactElement {
  const handOverPaths: THandOverPaths = useHandOverPaths();
  const { pathname } = useLocation();

  useToolBusy(isBusy);

  return (
    <>
      <span data-testid={"route"}>{pathname}</span>

      <button
        type={"button"}
        onClick={() => handOverPaths(EToolId.LEVEL_COMPILER, { source: "C:\\rawdata\\levels\\labx8.level" })}
      >
        Hand over
      </button>
    </>
  );
}

describe("useHandOverPaths", () => {
  beforeEach(() => {
    clearLocalStorage();
  });

  it("opens the other tool holding the paths handed over", async () => {
    const { getByTestId, getByText } = renderWithProviders(<HandOver isBusy={false} />, { route: "/level-editor" });

    await userEvent.click(getByText("Hand over"));

    expect(getByTestId("route")).toHaveTextContent("/level-compiler");
    expect(getLocalStorageValue(getFieldValueStorageKey(EToolId.LEVEL_COMPILER, "source"))).toBe(
      "C:\\rawdata\\levels\\labx8.level"
    );
  });

  it("stays, remembering nothing, while work running holds the tool", async () => {
    const { getByTestId, getByText } = renderWithProviders(<HandOver isBusy={true} />, { route: "/level-editor" });

    await userEvent.click(getByText("Hand over"));

    expect(getByTestId("route")).toHaveTextContent("/level-editor");
    expect(getLocalStorageValue(getFieldValueStorageKey(EToolId.LEVEL_COMPILER, "source"))).toBeNull();
  });
});
