import { ReactElement } from "react";

import { SpawnConversionResult } from "@/core/bindings/types/xrf-app";
import { CommandResult } from "@/core/ui/command-result/CommandResult";

interface ISpawnConversionOutcomeProps {
  result: SpawnConversionResult;
}

/** Shows what the backend wrote, including results restored after a reload. */
export function SpawnConversionOutcome({ result }: ISpawnConversionOutcomeProps): ReactElement {
  return (
    <CommandResult
      headline={
        result.outcome === "cancelled"
          ? "Stopped before writing. Output was left unchanged."
          : `${result.operation === "pack" ? "Packed" : "Unpacked"} spawn to ${result.destination}`
      }
      tone={result.outcome === "cancelled" ? "info" : "success"}
      stats={[]}
    />
  );
}
