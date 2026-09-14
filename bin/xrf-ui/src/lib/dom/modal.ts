/**
 * Whether a modal dialog is on screen.
 *
 * Asked globally rather than of the key event's target: a click on the backdrop can leave focus outside the dialog,
 * and a shortcut must not reach the application behind a modal in that moment either.
 *
 * @returns Whether anything on the page is marked as a modal.
 */
export function isModalOpen(): boolean {
  return document.querySelector('[aria-modal="true"]') !== null;
}
