import { IRenderInputEvent } from "#/contract/render-input-event";

/** What the forwarder does with a gesture it has taken. */
export type TRenderInputSink = (event: IRenderInputEvent) => void;
