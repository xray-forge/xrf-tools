import { TRendererResponse } from "#/contract/renderer-response";

/** Posts a response, moving what it names rather than copying it. */
export type TRendererReply = (response: TRendererResponse, transfers?: Array<Transferable>) => void;
