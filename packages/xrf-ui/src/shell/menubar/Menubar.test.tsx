import { describe, expect, it, jest } from "@jest/globals";
import { render, RenderResult } from "@testing-library/react";
import { userEvent } from "@testing-library/user-event";
import { ReactElement, useState } from "react";

import { Menubar } from "./Menubar";
import { IMenubarItem, IMenubarMenu, IMenubarSubmenu, TMenubarEntry } from "./menubar-menu";

function item(label: string, overrides: Partial<IMenubarItem> = {}): IMenubarItem {
  return { id: label.toLowerCase(), label, onSelect: jest.fn(), ...overrides };
}

function submenu(label: string, sections: ReadonlyArray<ReadonlyArray<TMenubarEntry>>): IMenubarSubmenu {
  return { id: label.toLowerCase(), label, sections };
}

function menu(label: string, sections: ReadonlyArray<ReadonlyArray<TMenubarEntry>>): IMenubarMenu {
  return { id: `test/${label.toLowerCase()}`, label, getSections: () => sections };
}

/** Mounts the bar on demand the way the caption does, with somewhere to hand the keyboard back to. */
function Host({ menus, onCollapse }: { menus: ReadonlyArray<IMenubarMenu>; onCollapse?: () => void }): ReactElement {
  const [isExpanded, setExpanded] = useState(false);

  return (
    <>
      <button onClick={() => setExpanded(true)}>Expand</button>

      {isExpanded ? (
        <Menubar
          label={"Application menu"}
          menus={menus}
          onCollapse={() => {
            onCollapse?.();
            setExpanded(false);
          }}
        />
      ) : null}
    </>
  );
}

async function renderExpanded(menus: ReadonlyArray<IMenubarMenu>, onCollapse?: () => void): Promise<RenderResult> {
  const result: RenderResult = render(<Host menus={menus} onCollapse={onCollapse} />);

  await userEvent.click(result.getByRole("button", { name: "Expand" }));

  return result;
}

describe("Menubar", () => {
  it("takes the keyboard as it appears and moves between menus with the arrows, wrapping at the ends", async () => {
    const { getByRole } = await renderExpanded([menu("File", [[item("Exit")]]), menu("Help", [[item("About")]])]);

    expect(getByRole("menuitem", { name: "File" })).toHaveFocus();

    await userEvent.keyboard("{ArrowRight}");
    expect(getByRole("menuitem", { name: "Help" })).toHaveFocus();

    await userEvent.keyboard("{ArrowRight}");
    expect(getByRole("menuitem", { name: "File" })).toHaveFocus();
  });

  it("opens a menu from the keyboard with its first row ready, and steps sideways into the next one", async () => {
    const { getByRole } = await renderExpanded([
      menu("File", [[item("Exit")]]),
      menu("Help", [[item("About")], [item("Source")]]),
    ]);

    await userEvent.keyboard("{ArrowDown}");
    expect(getByRole("menuitem", { name: "Exit" })).toHaveFocus();

    await userEvent.keyboard("{ArrowRight}");
    expect(getByRole("menu", { name: "Help" })).toBeInTheDocument();
    expect(getByRole("menuitem", { name: "About" })).toHaveFocus();
    // Sections draw apart, and the divider between them is not a stop for the caret.
    expect(getByRole("separator")).toBeInTheDocument();
  });

  it("switches menus on hover once one is open, as a desktop menubar does, and not before", async () => {
    const { getByRole, queryByRole } = await renderExpanded([
      menu("File", [[item("Exit")]]),
      menu("Help", [[item("About")]]),
    ]);

    await userEvent.hover(getByRole("menuitem", { name: "Help" }));
    expect(queryByRole("menu")).not.toBeInTheDocument();

    await userEvent.click(getByRole("menuitem", { name: "File" }));
    await userEvent.hover(getByRole("menuitem", { name: "Help" }));

    expect(getByRole("menu", { name: "Help" })).toBeInTheDocument();
    expect(queryByRole("menu", { name: "File" })).not.toBeInTheDocument();
  });

  it("puts the whole bar away on Escape from an open menu, handing the keyboard back to where it was", async () => {
    const onCollapse = jest.fn();
    const { getByRole, queryByRole } = await renderExpanded([menu("File", [[item("Exit")]])], onCollapse);

    await userEvent.keyboard("{ArrowDown}");
    await userEvent.keyboard("{Escape}");

    expect(onCollapse).toHaveBeenCalledTimes(1);
    expect(queryByRole("menubar")).not.toBeInTheDocument();
    expect(getByRole("button", { name: "Expand" })).toHaveFocus();
  });

  it("closes when something outside it is pressed, even where the press is kept from becoming a mouse event", async () => {
    const onCollapse = jest.fn();
    const { getByRole } = await renderExpanded([menu("File", [[item("Exit")]])], onCollapse);
    const viewport: HTMLElement = document.createElement("div");

    // The viewport cancels its pointer presses, so the browser raises no mousedown for them.
    viewport.addEventListener("pointerdown", (event: Event) => event.preventDefault());
    document.body.appendChild(viewport);

    await userEvent.click(getByRole("menuitem", { name: "File" }));
    await userEvent.pointer({ keys: "[MouseLeft]", target: viewport });

    expect(onCollapse).toHaveBeenCalledTimes(1);
  });

  it("puts the bar away before a chosen row runs, so the row finds no menu holding the keyboard", async () => {
    let isBarShownWhenRun: boolean = true;
    const exit: IMenubarItem = item("Exit", {
      onSelect: jest.fn(() => {
        isBarShownWhenRun = document.querySelector('[role="menubar"]') !== null;
      }),
    });
    const { getByRole } = await renderExpanded([menu("File", [[exit]])]);

    await userEvent.click(getByRole("menuitem", { name: "File" }));
    await userEvent.click(getByRole("menuitem", { name: "Exit" }));

    expect(exit.onSelect).toHaveBeenCalledTimes(1);
    expect(isBarShownWhenRun).toBe(false);
  });

  it("keeps the caret in the bar when the menu holding it leaves while the bar is open", async () => {
    const file: IMenubarMenu = menu("File", [[item("Exit")]]);
    const { getByRole, queryByRole, rerender } = await renderExpanded([file, menu("Sprite", [[item("Reload")]])]);

    await userEvent.keyboard("{ArrowRight}");
    expect(getByRole("menuitem", { name: "Sprite" })).toHaveFocus();

    // The tool publishing Sprite went away, and its button with it.
    rerender(<Host menus={[file]} />);

    expect(getByRole("menuitem", { name: "File" })).toHaveFocus();

    await userEvent.keyboard("{Escape}");

    expect(queryByRole("menubar")).not.toBeInTheDocument();
  });

  it("draws a switch with its state and a disabled row that cannot be chosen", async () => {
    const disabled: IMenubarItem = item("Reload", { isDisabled: true, shortcut: "F5" });
    const { getByRole, getByText } = await renderExpanded([
      menu("View", [[item("Grid", { isChecked: true }), item("Axes", { isChecked: false }), disabled]]),
    ]);

    await userEvent.click(getByRole("menuitem", { name: "View" }));

    expect(getByRole("menuitemcheckbox", { name: "Grid" })).toHaveAttribute("aria-checked", "true");
    expect(getByRole("menuitemcheckbox", { name: "Axes" })).toHaveAttribute("aria-checked", "false");
    expect(getByRole("menuitem", { name: /Reload/ })).toHaveAttribute("aria-disabled", "true");
    expect(getByText("F5")).toBeInTheDocument();
  });

  it("opens a submenu from the keyboard with its first row ready, and closes it back onto its row", async () => {
    const { getByRole, queryByRole } = await renderExpanded([
      menu("View", [[submenu("Shading", [[item("Final", { isChecked: true, isExclusive: true })]])]]),
    ]);

    await userEvent.keyboard("{ArrowDown}");
    expect(getByRole("menuitem", { name: "Shading" })).toHaveFocus();

    await userEvent.keyboard("{ArrowRight}");
    expect(getByRole("menu", { name: "Shading" })).toBeInTheDocument();
    expect(getByRole("menuitemradio", { name: "Final" })).toHaveFocus();
    expect(getByRole("menuitem", { name: "Shading" })).toHaveAttribute("aria-expanded", "true");

    await userEvent.keyboard("{ArrowLeft}");
    expect(queryByRole("menu", { name: "Shading" })).not.toBeInTheDocument();
    expect(getByRole("menuitem", { name: "Shading" })).toHaveFocus();
  });

  it("closes only the submenu on Escape, leaving the menu that opened it", async () => {
    const onCollapse = jest.fn();
    const { getByRole, queryByRole } = await renderExpanded(
      [menu("View", [[submenu("Shading", [[item("Clay")]])]])],
      onCollapse
    );

    await userEvent.keyboard("{ArrowDown}{ArrowRight}{Escape}");

    expect(queryByRole("menu", { name: "Shading" })).not.toBeInTheDocument();
    expect(getByRole("menu", { name: "View" })).toBeInTheDocument();
    expect(onCollapse).not.toHaveBeenCalled();
  });

  it("steps to the next menu on the right arrow from a submenu's plain row", async () => {
    const { getByRole } = await renderExpanded([
      menu("View", [[submenu("Shading", [[item("Clay")]])]]),
      menu("Help", [[item("About")]]),
    ]);

    await userEvent.keyboard("{ArrowDown}{ArrowRight}{ArrowRight}");

    expect(getByRole("menu", { name: "Help" })).toBeInTheDocument();
    expect(getByRole("menuitem", { name: "About" })).toHaveFocus();
  });

  it("opens a submenu by pointing at its row, and chooses in it with the menus kept open for a switch", async () => {
    const clay: IMenubarItem = item("Clay", { isChecked: false, isExclusive: true });
    const onCollapse = jest.fn();
    const { getByRole, findByRole } = await renderExpanded(
      [menu("View", [[submenu("Shading", [[item("Final", { isChecked: true, isExclusive: true }), clay]])]])],
      onCollapse
    );

    await userEvent.click(getByRole("menuitem", { name: "View" }));
    await userEvent.hover(getByRole("menuitem", { name: "Shading" }));
    await userEvent.click(await findByRole("menuitemradio", { name: "Clay" }));

    expect(clay.onSelect).toHaveBeenCalledTimes(1);
    expect(getByRole("menuitemradio", { name: "Final" })).toHaveAttribute("aria-checked", "true");
    expect(getByRole("menu", { name: "Shading" })).toBeInTheDocument();
    expect(onCollapse).not.toHaveBeenCalled();
  });

  it("puts the whole bar away when a submenu's action is chosen", async () => {
    const reset: IMenubarItem = item("Reset");
    const onCollapse = jest.fn();
    const { getByRole } = await renderExpanded([menu("View", [[submenu("Camera", [[reset]])]])], onCollapse);

    await userEvent.click(getByRole("menuitem", { name: "View" }));
    await userEvent.click(getByRole("menuitem", { name: "Camera" }));
    await userEvent.click(getByRole("menuitem", { name: "Reset" }));

    expect(reset.onSelect).toHaveBeenCalledTimes(1);
    expect(onCollapse).toHaveBeenCalledTimes(1);
  });

  it("keeps the menu open when a switch is turned, so several can be turned in one visit", async () => {
    const grid: IMenubarItem = item("Grid", { isChecked: false });
    const onCollapse = jest.fn();
    const { getByRole } = await renderExpanded([menu("View", [[grid]])], onCollapse);

    await userEvent.click(getByRole("menuitem", { name: "View" }));
    await userEvent.click(getByRole("menuitemcheckbox", { name: "Grid" }));

    expect(grid.onSelect).toHaveBeenCalledTimes(1);
    expect(onCollapse).not.toHaveBeenCalled();
    expect(getByRole("menu", { name: "View" })).toBeInTheDocument();
  });
});
