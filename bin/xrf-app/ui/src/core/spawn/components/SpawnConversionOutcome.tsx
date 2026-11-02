import { ESpawnConversion, SpawnConversionResult } from "@xrf/ipc/types/xrf-app-spawn";
import { EJobOutcome } from "@xrf/ipc/types/xrf-job";
import { CommandResult } from "@xrf/ui/command-result";
import { ReactElement } from "react";

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
