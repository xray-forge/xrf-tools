import { IRenderInputEvent } from "#/contract/render-input-event";

/** The event as it crossed, plus the two calls a control makes on one. */
export interface IRenderProxyEvent extends IRenderInputEvent {
  preventDefault(): void;
  stopPropagation(): void;
}
