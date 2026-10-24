// Auto-generated rust bindings. Do not edit it manually.

import { invoke as __TAURI_INVOKE } from "#/invoke";
import { IIpcCommandSignature } from "#/signature";
import { TransportEndpoint } from "#/types/xrf-app-core";

/** The wire name of every command of the `transport` plugin. */
export enum ETransportCommand {
  GET_ENDPOINT = "plugin:transport|get_endpoint",
}

/** Commands */
export const transportCommands = {
  /** Where the transport listens and the token it expects, which the frontend asks once. */
  getEndpoint: () => __TAURI_INVOKE<TransportEndpoint>(ETransportCommand.GET_ENDPOINT),
};

/** What each command of the `transport` plugin takes and resolves to, keyed by its wire name. */
export interface ITransportCommandSignatures {
  [ETransportCommand.GET_ENDPOINT]: IIpcCommandSignature<typeof transportCommands.getEndpoint, []>;
}
