import { useMemo } from "react";

import { Logger } from "./Logger";

export function useLogger(prefix: string, isEnabled: boolean = true): Logger {
  return useMemo(() => new Logger(prefix, isEnabled), [prefix, isEnabled]);
}
