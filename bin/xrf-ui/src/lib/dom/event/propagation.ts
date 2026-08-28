import { MouseEvent } from "react";

export function stopPropagation(event: MouseEvent<HTMLElement>): void {
  event.stopPropagation();
}

export function preventDefault(event: MouseEvent<HTMLElement>): void {
  event.preventDefault();
}

/**
 * Wraps an action so the click that runs it goes no further.
 *
 * For a control sitting inside something clickable of its own, where letting the event through would run both: a
 * button inside a row that is itself a button, for instance.
 *
 * @param action - What the control does.
 * @returns A handler that stops the event and then runs the action.
 */
export function withStoppedPropagation(action: () => void): (event: MouseEvent<HTMLElement>) => void {
  return (event: MouseEvent<HTMLElement>) => {
    event.stopPropagation();
    action();
  };
}
