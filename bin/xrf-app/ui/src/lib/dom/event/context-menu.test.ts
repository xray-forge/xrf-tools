import { afterEach, describe, expect, it } from "@jest/globals";

import { suppressNativeContextMenu } from "@/lib/dom/event/context-menu";

const disposers: Array<() => void> = [];

function suppress(): void {
  disposers.push(suppressNativeContextMenu());
}

function mount<T extends HTMLElement>(element: T): T {
  document.body.appendChild(element);

  return element;
}

/**
 * Asks for a context menu the way a right click does.
 *
 * @param target - Element to right click.
 * @returns Whether the webview would have been told to show its own menu.
 */
function isWebviewMenuShown(target: HTMLElement): boolean {
  const event: MouseEvent = new MouseEvent("contextmenu", { bubbles: true, cancelable: true });

  target.dispatchEvent(event);

  return !event.defaultPrevented;
}

afterEach(() => {
  disposers.splice(0).forEach((dispose) => dispose());
  document.body.innerHTML = "";
});

describe("suppressNativeContextMenu", () => {
  it("suppresses the webview menu anywhere in the document", () => {
    suppress();

    expect(isWebviewMenuShown(mount(document.createElement("div")))).toBe(false);
  });

  it("leaves the webview menu to text fields, which have no other clipboard UI", () => {
    suppress();

    expect(isWebviewMenuShown(mount(document.createElement("input")))).toBe(true);
    expect(isWebviewMenuShown(mount(document.createElement("textarea")))).toBe(true);
  });

  it("leaves the webview menu to an editable region and anything inside it", () => {
    const editable: HTMLDivElement = mount(document.createElement("div"));
    const child: HTMLSpanElement = editable.appendChild(document.createElement("span"));

    editable.setAttribute("contenteditable", "true");
    suppress();

    expect(isWebviewMenuShown(editable)).toBe(true);
    expect(isWebviewMenuShown(child)).toBe(true);
  });

  it("suppresses the webview menu for an explicitly non editable region", () => {
    const element: HTMLDivElement = mount(document.createElement("div"));

    element.setAttribute("contenteditable", "false");
    suppress();

    expect(isWebviewMenuShown(element)).toBe(false);
  });

  it("restores the webview menu once disposed", () => {
    const element: HTMLDivElement = mount(document.createElement("div"));
    const dispose: () => void = suppressNativeContextMenu();

    expect(isWebviewMenuShown(element)).toBe(false);

    dispose();

    expect(isWebviewMenuShown(element)).toBe(true);
  });

  it("hands the gesture back when an element handler stops propagation", () => {
    // Documents the trap the suppressor cannot cover: a custom menu has to prevent the default itself.
    const element: HTMLDivElement = mount(document.createElement("div"));

    element.addEventListener("contextmenu", (event) => event.stopPropagation());
    suppress();

    expect(isWebviewMenuShown(element)).toBe(true);
  });
});
