import { useMemo } from "react";

import { Logger } from "#/logger";

export function useLogger(prefix: string, isEnabled: boolean = true): Logger {
  return useMemo(() => new Logger(prefix, isEnabled), [prefix, isEnabled]);
}
