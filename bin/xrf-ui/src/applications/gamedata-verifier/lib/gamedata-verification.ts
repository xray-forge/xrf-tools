import { GamedataVerifyRequest } from "@/core/ipc/types/xrf-app";

/** What a verification of a whole gamedata root is asked to do: every check, so the request's selection is left out. */
export interface IGamedataVerification extends Omit<GamedataVerifyRequest, "checks"> {}
