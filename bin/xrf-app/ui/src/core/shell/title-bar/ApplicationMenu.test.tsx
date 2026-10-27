import { beforeEach, describe, expect, it, jest } from "@jest/globals";
import { act, RenderResult, within } from "@testing-library/react";
import { userEvent } from "@testing-library/user-event";
import { ReactElement } from "react";
import { useLocation } from "react-router-dom";

import { NotificationsService } from "@/core/notifications/services";
import { ApplicationShell } from "@/core/shell/ApplicationShell";
import { useShellMenus } from "@/core/shell/contributions";
import { useEditorBusy, useEditorDirty } from "@/core/shell/editor-lifecycle";
import { IMenuContribution } from "@/core/shell/menu/lib";
import { clearLocalStorage } from "@/core/storage";
import { renderWithProviders } from "@/fixtures/utils/render";
import { SPRITE_EQUIPMENT_EDITOR_MENU } from "@/tools/sprite-equipment-editor/menus";

const PLANNED_ROUTE: string = "/level-compiler";

function Location({ dirtyCount = 0, isBusy = false }: { dirtyCount?: number; isBusy?: boolean }): ReactElement {
  const { pathname } = useLocation();

  useEditorDirty(dirtyCount);
  useEditorBusy(isBusy);

  return <div>at {pathname}</div>;
}

function SpriteMenuPublisher({ onRepack }: { onRepack: () => void }): ReactElement {
  useShellMenus(
    (): Array<IMenuContribution> => [
      {
        section: SPRITE_EQUIPMENT_EDITOR_MENU.sections.sprite,
        entries: [{ kind: "action", id: "sprite-equipment-editor/repack", label: "Repack", onSelect: onRepack }],
      },
    ],
    [onRepack]
  );

  return <div>sprite editor</div>;
}

function renderShell(content: ReactElement, route: string): RenderResult {
  return renderWithProviders(<ApplicationShell>{content}</ApplicationShell>, {
    bindings: [NotificationsService],
    hasShell: true,
    route,
  });
}

async function openMenu(result: RenderResult, name: string): Promise<void> {
  // The click opens File itself; any other menu is one more click along the bar.
  await userEvent.click(result.getByRole("button", { name: "Menu" }));

  if (name !== "File") {
    // Pointing at another menu while one is open switches to it; a click there would close it again.
    await userEvent.hover(result.getByRole("menuitem", { name }));
  }
}

describe("ApplicationMenu", () => {
  beforeEach(() => clearLocalStorage());

  it("opens File at once when the menu button is clicked", async () => {
    const result: RenderResult = renderShell(<Location />, PLANNED_ROUTE);

    await userEvent.click(result.getByRole("button", { name: "Menu" }));

    expect(result.getByRole("menu", { name: "File" })).toBeInTheDocument();
    // Inside the caption, which is what hides the breadcrumb under the bar without moving the toolbar.
    expect(within(result.getByTestId("application-title-bar")).getByRole("menubar")).toBeInTheDocument();
  });

  it("closes the open tool from File and goes back to all tools", async () => {
    const result: RenderResult = renderShell(<Location />, PLANNED_ROUTE);

    await openMenu(result, "File");
    await userEvent.click(result.getByRole("menuitem", { name: /^Close/ }));

    expect(await result.findByText("at /")).toBeInTheDocument();
    expect(result.queryByRole("menubar")).not.toBeInTheDocument();
  });

  it("asks before closing a tool that holds unsaved work", async () => {
    const result: RenderResult = renderShell(<Location dirtyCount={2} />, PLANNED_ROUTE);

    await openMenu(result, "File");
    await userEvent.click(result.getByRole("menuitem", { name: /^Close/ }));

    expect(result.getByText("Leave without saving?")).toBeInTheDocument();
    expect(result.getByText(`at ${PLANNED_ROUTE}`)).toBeInTheDocument();
  });

  it("offers Close only while a tool is open", async () => {
    const result: RenderResult = renderShell(<Location />, "/");

    await openMenu(result, "File");

    expect(result.getByRole("menuitem", { name: /^Close/ })).toHaveAttribute("aria-disabled", "true");
    expect(result.getByRole("menuitem", { name: /^Settings/ })).not.toHaveAttribute("aria-disabled");
  });

  it("offers no Close while the open tool is busy, since leaving would strand what it runs", async () => {
    const result: RenderResult = renderShell(<Location isBusy />, PLANNED_ROUTE);

    await openMenu(result, "File");

    expect(result.getByRole("menuitem", { name: /^Close/ })).toHaveAttribute("aria-disabled", "true");
  });

  it("draws a tool's own menus between File and Help, holding what the tool publishes", async () => {
    const onRepack = jest.fn();
    // The tool's services load before its content, as they do in the window.
    const result: RenderResult = await act(async () =>
      renderShell(<SpriteMenuPublisher onRepack={onRepack} />, "/sprite-equipment-editor")
    );

    await result.findByText("sprite editor");
    await userEvent.click(result.getByRole("button", { name: "Menu" }));

    const bar: HTMLElement = result.getByRole("menubar");

    expect(
      within(bar)
        .getAllByRole("menuitem")
        .map((it: HTMLElement) => it.textContent)
    ).toEqual(["File", "Sprite", "Help"]);

    await userEvent.hover(within(bar).getByRole("menuitem", { name: "Sprite" }));
    await userEvent.click(result.getByRole("menuitem", { name: "Repack" }));

    expect(onRepack).toHaveBeenCalledTimes(1);
  });

  it("opens from Alt pressed on its own, and not from Alt held through a click", async () => {
    const result: RenderResult = renderShell(<Location />, PLANNED_ROUTE);

    await userEvent.keyboard("{Alt>}");
    await userEvent.click(result.getByText(`at ${PLANNED_ROUTE}`));
    await userEvent.keyboard("{/Alt}");

    expect(result.queryByRole("menubar")).not.toBeInTheDocument();

    await userEvent.keyboard("{Alt}");

    expect(result.getByRole("menubar")).toBeInTheDocument();
    expect(result.getByRole("menuitem", { name: "File" })).toHaveFocus();
  });

  it("lets no shortcut reach the tool while the menubar holds the keyboard", async () => {
    const result: RenderResult = renderShell(<Location />, PLANNED_ROUTE);

    await userEvent.keyboard("{F10}");
    await userEvent.keyboard("{Control>}w{/Control}");

    expect(result.getByText(`at ${PLANNED_ROUTE}`)).toBeInTheDocument();

    await userEvent.keyboard("{Escape}");
    await userEvent.keyboard("{Control>}w{/Control}");

    expect(await result.findByText("at /")).toBeInTheDocument();
  });

  it("opens settings at About from Help", async () => {
    const result: RenderResult = renderShell(<Location />, PLANNED_ROUTE);

    await openMenu(result, "Help");
    await userEvent.click(result.getByRole("menuitem", { name: "About" }));

    const dialog: HTMLElement = result.getByRole("dialog", { name: "Settings" });

    expect(within(dialog).getByRole("button", { name: "About" })).toHaveClass("Mui-selected");
  });
});
