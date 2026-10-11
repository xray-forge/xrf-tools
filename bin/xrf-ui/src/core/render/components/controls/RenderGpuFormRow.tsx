import { Nullable } from "@xrf/types";
import { ReactElement } from "react";

import { RenderBackendAvailability, RenderGpu, RenderGpuDescription } from "@/core/ipc/types/xrf-renderer";
import { resolveRenderBackend, TRenderBackendChoice } from "@/core/render/lib/settings/render-backend-choice";
import {
  describeRenderGpu,
  fromRenderGpuKey,
  RENDER_GPU_AUTO,
  resolveRenderGpu,
  toRenderGpuKey,
  TRenderGpuKey,
} from "@/core/render/lib/settings/render-gpu-choice";
import { ChoiceFormRow, IChoiceFormRowOption } from "@/core/ui/form/ChoiceFormRow";
import { BaseComponentProps } from "@/lib/dom/element-types";

/**
 * @param gpu - The GPU the settings ask for, or null for the automatic one.
 * @param backend - The graphics API drawn with, as the renderer probed it; none until known.
 * @returns What the row says: which GPU draws, and why the one asked for does not where it does not.
 */
function describeGpuChoice(gpu: Nullable<RenderGpu>, backend: RenderBackendAvailability | undefined): string {
  const lead: string =
    "The GPU every viewport draws on. Automatic takes the graphics API's high-performance GPU; changing it restarts " +
    "the renderer and streams the level in again.";
  const drawn: Nullable<RenderGpuDescription> = resolveRenderGpu(gpu, backend);

  if (backend === undefined || drawn === null) {
    return lead;
  }

  const drawnOn: string = `Drawing on ${describeRenderGpu(drawn)}.`;

  return gpu !== null && toRenderGpuKey(gpu) !== toRenderGpuKey(drawn.gpu)
    ? `${lead} The chosen GPU is not available, so the renderer uses the automatic one. ${drawnOn}`
    : `${lead} ${drawnOn}`;
}

export interface IRenderGpuFormRowProps extends BaseComponentProps {
  /** The GPU the settings ask for, or null for the automatic one. */
  value: Nullable<RenderGpu>;
  /** The graphics API the settings ask for, which decides the GPUs offered. */
  backend: TRenderBackendChoice;
  /** What each graphics API can do on this machine, none until the renderer was asked. */
  availability: Nullable<ReadonlyArray<RenderBackendAvailability>>;
  onChange: (gpu: Nullable<RenderGpu>) => void;
}

/**
 * The settings' choice of GPU: automatic, or one of the GPUs the graphics API drawn with can use; says which one draws
 * when the one chosen is missing and the renderer fell back.
 */
export function RenderGpuFormRow({
  "data-testid": dataTestId = "render-gpu-form-row",
  id,
  className,
  value,
  backend,
  availability,
  onChange,
}: IRenderGpuFormRowProps): ReactElement {
  const drawn: RenderBackendAvailability | undefined =
    availability === null
      ? undefined
      : availability.find(
          (it: RenderBackendAvailability) => it.backend === resolveRenderBackend(backend, availability)
        );
  const gpus: ReadonlyArray<RenderGpuDescription> = drawn?.gpus ?? [];
  const options: ReadonlyArray<IChoiceFormRowOption<TRenderGpuKey>> = [
    { label: "Automatic", value: RENDER_GPU_AUTO },
    ...gpus.map((it: RenderGpuDescription) => ({ label: it.name, value: toRenderGpuKey(it.gpu) })),
  ];

  return (
    <ChoiceFormRow
      data-testid={dataTestId}
      id={id}
      className={className}
      label={"GPU"}
      description={describeGpuChoice(value, drawn)}
      options={options}
      value={toRenderGpuKey(value)}
      onChange={(key: TRenderGpuKey) => onChange(fromRenderGpuKey(key, gpus))}
    />
  );
}
