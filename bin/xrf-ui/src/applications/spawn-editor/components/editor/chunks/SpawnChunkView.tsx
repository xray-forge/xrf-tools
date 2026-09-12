import { Box } from "@mui/material";
import { ReactElement, ReactNode } from "react";

import { DelayedProgress } from "@/core/ui/layout/DelayedProgress";
import { EmptyState } from "@/core/ui/layout/EmptyState";
import { ErrorState } from "@/core/ui/layout/ErrorState";
import { AsyncState } from "@/lib/async-state";
import { useMountEffect } from "@/lib/react";
import { Nullable } from "@/lib/types/general";

export interface ISpawnChunkViewProps<T> {
  chunk: AsyncState<Nullable<T>>;
  /**
   * Loads a lazy chunk on mount or retry. Omitted for the header supplied by the session.
   */
  onLoad?: () => void;
  render: (value: T) => ReactNode;
}

/**
 * The frame every spawn chunk renders into.
 */
export function SpawnChunkView<T>({ chunk, onLoad, render }: ISpawnChunkViewProps<T>): ReactElement {
  useMountEffect(() => void onLoad?.());

  if (chunk.isLoading) {
    return <DelayedProgress label={"Reading spawn chunk…"} />;
  }

  if (chunk.error) {
    return <ErrorState title={"Could not read this chunk"} description={chunk.error.message} onRetry={onLoad} />;
  }

  if (!chunk.value) {
    return <EmptyState title={"Nothing to show"} description={"Open a spawn file to read its chunks."} />;
  }

  return (
    <Box
      sx={{
        display: "flex",
        flexDirection: "column",
        width: "100%",
        height: "100%",
        minHeight: 0,
        padding: 2,
        flexWrap: "nowrap",
      }}
    >
      {render(chunk.value)}
    </Box>
  );
}
