import { Nullable } from "@xrf/types";
import { ReactElement } from "react";

import { RenderBackend, RenderBackendAvailability } from "@/core/ipc/types/xrf-renderer";
import {
  describeRenderBackend,
  isRenderBackendAvailable,
  RENDER_BACKEND_AUTO,
  RENDER_BACKEND_CHOICES,
  resolveRenderBackend,
  TRenderBackendChoice,
} from "@/core/render/lib/settings/render-backend-choice";
import { ChoiceFormRow, IChoiceFormRowOption } from "@/core/ui/form/ChoiceFormRow";
import { BaseComponentProps } from "@/lib/dom/element-types";

/**
 * @param value - The graphics API the settings ask for.
 * @param availability - What each graphics API can do here, none until known.
 * @returns What the row says: which API draws, on which GPU, and why the one asked for does not where it does not.
 */
function describeBackendChoice(
  value: TRenderBackendChoice,
  availability: Nullable<ReadonlyArray<RenderBackendAvailability>>
): string {
  const lead: string =
    "The API every viewport draws with. Automatic takes the first that works; moving to another API restarts the " +
    "renderer and streams the level in again.";

  if (availability === null) {
    return lead;
  }

  const drawn: Nullable<RenderBackend> = resolveRenderBackend(value, availability);

  if (drawn === null) {
    return `${lead} No graphics API works on this machine.`;
  }

  const adapter: Nullable<string> =
    availability.find((it: RenderBackendAvailability) => it.backend === drawn)?.adapter ?? null;
  const drawnOn: string = `Drawing with ${describeRenderBackend(drawn)} on ${adapter ?? "the GPU"}.`;

  if (value !== RENDER_BACKEND_AUTO && value !== drawn) {
    return `${lead} ${describeRenderBackend(value)} is not available here, so the renderer fell back. ${drawnOn}`;
  }

  return `${lead} ${drawnOn}`;
}

export interface IRenderBackendFormRowProps extends BaseComponentProps {
  /** The graphics API the settings ask for. */
  value: TRenderBackendChoice;
  /** What each graphics API can do on this machine, none until the renderer was asked. */
  availability: Nullable<ReadonlyArray<RenderBackendAvailability>>;
  onChange: (choice: TRenderBackendChoice) => void;
}

/**
 * The settings' choice of graphics API: automatic, or one by name where this machine can draw with it, the others
 * greyed out; says which one is drawn with when the one asked for is not available and the renderer fell back.
 */
export function RenderBackendFormRow({
  "data-testid": dataTestId = "render-backend-form-row",
  id,
  className,
  value,
  availability,
  onChange,
}: IRenderBackendFormRowProps): ReactElement {
  const options: ReadonlyArray<IChoiceFormRowOption<TRenderBackendChoice>> = RENDER_BACKEND_CHOICES.map(
    (choice: TRenderBackendChoice) =>
      choice === RENDER_BACKEND_AUTO
        ? { label: "Automatic", value: choice }
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
      description={describeBackendChoice(value, availability)}
      options={options}
      value={value}
      onChange={onChange}
    />
  );
}
