import { ELogLevel } from "#/logger.types";

/** Each level's verbosity: a line is kept where its rank is at most the rank of the level asked for. */
export const LOG_LEVEL_RANK: Readonly<Record<ELogLevel, number>> = {
  [ELogLevel.ERROR]: 0,
  [ELogLevel.WARN]: 1,
  [ELogLevel.INFO]: 2,
  [ELogLevel.DEBUG]: 3,
};
