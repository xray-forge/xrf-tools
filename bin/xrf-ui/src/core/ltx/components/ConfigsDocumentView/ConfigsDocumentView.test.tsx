import { describe, expect, it, jest } from "@jest/globals";
import { act, RenderResult } from "@testing-library/react";
import { userEvent } from "@testing-library/user-event";
import { Container } from "@wirestate/core";
import { runInAction } from "@wirestate/mobx";

import { ConfigsDocumentService } from "@/core/ltx/services/document";
import { ConfigsFindingsService } from "@/core/ltx/services/findings";
import { ConfigsProjectService } from "@/core/ltx/services/project";
import { ConfigsResolvedService } from "@/core/ltx/services/resolved";
import { ConfigsSchemeService } from "@/core/ltx/services/scheme";
import { mockContainer } from "@/fixtures/utils/container";
import { renderWithProviders } from "@/fixtures/utils/render";

import { ConfigsDocumentView } from "./ConfigsDocumentView";

const PATH: string = "configs/items/w_empty.ltx";

function renderView(onDeselect: () => void = jest.fn()): { render: RenderResult; service: ConfigsDocumentService } {
  const container: Container = mockContainer([
    ConfigsProjectService,
    ConfigsDocumentService,
    ConfigsFindingsService,
    ConfigsResolvedService,
    ConfigsSchemeService,
  ]);

  return {
    render: renderWithProviders(<ConfigsDocumentView onDeselect={onDeselect} />, { container }),
    service: container.get(ConfigsDocumentService),
  };
}

describe("ConfigsDocumentView", () => {
  it("says nothing is open until a config is selected", () => {
    const { render: render_ } = renderView();

    expect(render_.getByText("No config open")).toBeInTheDocument();
    expect(render_.queryByTestId("configs-document-header")).not.toBeInTheDocument();
  });

  it("names the selected config before its text has arrived", () => {
    const { render: render_, service } = renderView();

    act(() => runInAction(() => (service.selected = PATH)));

    expect(render_.getByTestId("configs-document-header")).toHaveTextContent(PATH);
  });

  it("hands the close action back to the caller, which owns what a selection holds", async () => {
    const onDeselect = jest.fn();
    const { render: render_, service } = renderView(onDeselect);

    act(() => runInAction(() => (service.selected = PATH)));

    await userEvent.click(render_.getByRole("button", { name: "Close config" }));

    expect(onDeselect).toHaveBeenCalledTimes(1);
  });
});
