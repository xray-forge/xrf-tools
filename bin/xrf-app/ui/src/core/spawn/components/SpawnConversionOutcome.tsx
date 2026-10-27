import { ESpawnConversion, SpawnConversionResult } from "@xrf/ipc/types/xrf-app-spawn";
import { EJobOutcome } from "@xrf/ipc/types/xrf-job";
import { ReactElement } from "react";

import { CommandResult } from "@/core/ui/command-result/CommandResult";

interface ISpawnConversionOutcomeProps {
  result: SpawnConversionResult;
}

/** Shows what the backend wrote, including results restored after a reload. */
export function SpawnConversionOutcome({ result }: ISpawnConversionOutcomeProps): ReactElement {
  const isCancelled: boolean = result.outcome === EJobOutcome.CANCELLED;

  return (
    <CommandResult
      headline={
        isCancelled
          ? "Stopped before writing. Output was left unchanged."
          : `${result.operation === ESpawnConversion.PACK ? "Packed" : "Unpacked"} spawn to ${result.destination}`
      }
      tone={isCancelled ? "info" : "success"}
      stats={[]}
    />
  );
}
