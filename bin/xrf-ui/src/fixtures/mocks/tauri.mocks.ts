import { jest } from "@jest/globals";

/** The part of Tauri's close event a page touches. */
export interface IMockCloseRequestedEvent {
  preventDefault: () => void;
}

type TMockCloseListener = (event: IMockCloseRequestedEvent) => void | Promise<void>;

const windowState: { isMaximized: boolean; listeners: Array<() => void>; closeListeners: Array<TMockCloseListener> } = {
  isMaximized: false,
  listeners: [],
  closeListeners: [],
};

/** Provide the mocked Tauri window used by component tests. */
export const mockAppWindow = {
  label: "main",
  isMaximized: jest.fn(async (): Promise<boolean> => windowState.isMaximized),
  minimize: jest.fn(async (): Promise<void> => undefined),
  close: jest.fn(async (): Promise<void> => undefined),
  toggleMaximize: jest.fn(async (): Promise<void> => setMockWindowMaximized(!windowState.isMaximized)),
  onCloseRequested: jest.fn(async (handler: TMockCloseListener): Promise<() => void> => {
    windowState.closeListeners.push(handler);

    return () => {
      windowState.closeListeners = windowState.closeListeners.filter((it: TMockCloseListener) => it !== handler);
    };
  }),
  onResized: jest.fn(async (handler: () => void): Promise<() => void> => {
    windowState.listeners.push(handler);

    return () => {
      windowState.listeners = windowState.listeners.filter((it: () => void) => it !== handler);
    };
  }),
};

/**
 * Sets the mocked window maximized state and notifies listeners.
 *
 * @param next - Whether the mocked window is maximized.
 */
export function setMockWindowMaximized(next: boolean): void {
  windowState.isMaximized = next;

  for (const listener of windowState.listeners) {
    listener();
  }
}

/**
 * Asks the mocked window to close the way the caption button or `Alt+F4` does, through the page's listeners.
 *
 * @returns Whether a listener held the close, which the real window answers by staying open.
 */
export async function requestMockWindowClose(): Promise<boolean> {
  let isPrevented: boolean = false;

  for (const listener of [...windowState.closeListeners]) {
    await listener({ preventDefault: () => (isPrevented = true) });
  }

  return isPrevented;
}

/**
 * Restores the mocked window to its initial state.
 *
 */
export function resetMockAppWindow(): void {
  windowState.isMaximized = false;
  windowState.listeners = [];
  windowState.closeListeners = [];
}
