import { AsyncState } from "@xrf/async";
import { useMountEffect } from "@xrf/react/mount-effect";
import { Nullable } from "@xrf/types";
import { BaseComponentProps } from "@xrf/ui/element-types";
import { ContentState } from "@xrf/ui/layout/content-state";
import { cn } from "@xrf/ui/theme";
import { ReactElement, ReactNode } from "react";

export interface ISpawnChunkViewProps<T> extends BaseComponentProps {
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
export function SpawnChunkView<T>({
  "data-testid": dataTestId = "spawn-chunk-view",
  id,
  className,
  chunk,
  onLoad,
  render,
}: ISpawnChunkViewProps<T>): ReactElement {
  useMountEffect(() => void onLoad?.());

  return (
    <ContentState<T>
      data-testid={dataTestId}
      id={id}
      className={className}
      value={chunk.value}
      isLoading={chunk.isLoading}
      error={chunk.error}
      loadingLabel={"Reading spawn chunk…"}
      errorTitle={"Could not read this chunk"}
      onRetry={onLoad}
      emptyTitle={"Nothing to show"}
      emptyDescription={"Open a spawn file to read its chunks."}
    >
      {(value: T) => (
        <div
          data-testid={dataTestId}
          id={id}
          className={cn("flex h-full min-h-0 w-full flex-col flex-nowrap p-4", className)}
        >
          {render(value)}
        </div>
      )}
    </ContentState>
  );
}
