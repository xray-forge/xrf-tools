import { describe, expect, it, jest } from "@jest/globals";
import { act, RenderResult } from "@testing-library/react";
import { userEvent } from "@testing-library/user-event";
import { Container } from "@wirestate/core";
import { runInAction } from "@wirestate/mobx";

import { ConfigsDocumentService } from "@/applications/configs-explorer/services/document";
import { ConfigsFindingsService } from "@/applications/configs-explorer/services/findings";
import { ConfigsProjectService } from "@/applications/configs-explorer/services/project";
import { ConfigsResolvedService } from "@/applications/configs-explorer/services/resolved";
import { ConfigsSchemeService } from "@/applications/configs-explorer/services/scheme";
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
    const {
      render: { getByText, queryByTestId },
    } = renderView();

    expect(getByText("No config open")).toBeInTheDocument();
    expect(queryByTestId("configs-document-header")).not.toBeInTheDocument();
  });

  it("names the selected config before its text has arrived", () => {
    const {
      render: { getByTestId },
      service,
    } = renderView();

    act(() => runInAction(() => (service.selected = PATH)));

    expect(getByTestId("configs-document-header")).toHaveTextContent(PATH);
  });

  it("hands the close action back to the caller, which owns what a selection holds", async () => {
    const onDeselect = jest.fn();
    const {
      render: { getByRole },
      service,
    } = renderView(onDeselect);

    act(() => runInAction(() => (service.selected = PATH)));

    await userEvent.click(getByRole("button", { name: "Close config" }));

    expect(onDeselect).toHaveBeenCalledTimes(1);
  });
});
