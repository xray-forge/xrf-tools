import { RenderBackend, RenderBackendAvailability, RenderGpu } from "@xrf/ipc/types/xrf-renderer";
import { Nullable } from "@xrf/types";
import { BaseComponentProps } from "@xrf/ui/element-types";
import { ReactElement } from "react";

import {
  describeRenderBackend,
  describeRenderDrawing,
  getForcedRenderBackend,
  IRenderDrawing,
  isRenderBackendAvailable,
  listRenderBackendChoices,
  RENDER_BACKEND_AUTO,
  resolveRenderDrawing,
  TRenderBackendChoice,
} from "@/core/render/lib/settings/render-backend-choice";
import { ChoiceFormRow, IChoiceFormRowOption } from "@/core/ui/form/ChoiceFormRow";

/**
 * @param value - The graphics API the settings ask for.
 * @param gpu - The GPU the settings ask for, or null for the automatic one.
 * @param availability - What each graphics API can do here, none until known.
 * @returns What the row says: which API draws, on which GPU, and why the one asked for does not where it does not.
 */
function describeBackendChoice(
  value: TRenderBackendChoice,
  gpu: Nullable<RenderGpu>,
  availability: Nullable<ReadonlyArray<RenderBackendAvailability>>
): string {
  const lead: string =
    "The API every viewport draws with. Auto takes the first that works; moving to another API restarts the " +
    "renderer and reads the level in again.";

  if (availability === null) {
    return lead;
  }

  const drawing: Nullable<IRenderDrawing> = resolveRenderDrawing(value, gpu, availability);

  if (drawing === null) {
    return `${lead} No graphics API works on this machine.`;
  }

  const drawn: RenderBackend = drawing.backend;
  const drawnOn: string = `Drawing with ${describeRenderDrawing(drawing)}.`;
  const forced: Nullable<RenderBackend> = getForcedRenderBackend(availability);

  if (forced !== null) {
    return forced === drawn
      ? `${lead} XRF_RENDER_BACKEND overrides this choice with ${describeRenderBackend(forced)}. ${drawnOn}`
      : `${lead} XRF_RENDER_BACKEND asks for ${describeRenderBackend(forced)}, but it is not available here, so the ` +
          `renderer fell back. ${drawnOn}`;
  }

  if (value !== RENDER_BACKEND_AUTO && value !== drawn) {
    return `${lead} ${describeRenderBackend(value)} is not available here, so the renderer fell back. ${drawnOn}`;
  }

  return `${lead} ${drawnOn}`;
}

export interface IRenderBackendFormRowProps extends BaseComponentProps {
  /** The graphics API the settings ask for. */
  value: TRenderBackendChoice;
  /** The GPU the settings ask for, or null for the automatic one. */
  gpu: Nullable<RenderGpu>;
  /** What each graphics API can do on this machine, none until the renderer was asked. */
  availability: Nullable<ReadonlyArray<RenderBackendAvailability>>;
  onChange: (choice: TRenderBackendChoice) => void;
}

/**
 * The settings' choice of graphics API: automatic, or one of this platform's by name where this machine can draw with it,
 * the others greyed out; says which one is drawn with when the one asked for is not available and the renderer fell back.
 */
export function RenderBackendFormRow({
  "data-testid": dataTestId = "render-backend-form-row",
  id,
  className,
  value,
  gpu,
  availability,
  onChange,
}: IRenderBackendFormRowProps): ReactElement {
  const options: ReadonlyArray<IChoiceFormRowOption<TRenderBackendChoice>> = listRenderBackendChoices(
    availability,
    value
  ).map((choice: TRenderBackendChoice) =>
    choice === RENDER_BACKEND_AUTO
      ? { label: "Auto", value: choice }
      : {
          isDisabled: availability !== null && !isRenderBackendAvailable(choice, availability),
          label: describeRenderBackend(choice),
          value: choice,
        }
  );

  return (
    <ChoiceFormRow
      data-testid={dataTestId}
      id={id}
      className={className}
      label={"Graphics API"}
      description={describeBackendChoice(value, gpu, availability)}
      options={options}
      value={value}
      onChange={onChange}
    />
  );
}
