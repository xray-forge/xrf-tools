/** What holds the keyboard while on screen: a modal dialog, an open menu, or an expanded menubar. */
const MODAL_SELECTOR: string = '[aria-modal="true"], [role="menu"], [role="menubar"]';

/**
 * Whether a modal dialog or a menu is on screen, asked globally since a backdrop click can leave focus outside it.
 *
 * @returns Whether anything on the page is marked as a modal, a menu or a menubar.
 */
export function isModalOpen(): boolean {
  return document.querySelector(MODAL_SELECTOR) !== null;
}
