import { formatNumber } from "@xrf/format";
import { ArchiveLevelEnvModifier } from "@xrf/ipc/types/xrf-app-archives";
import { EditorPanelProperty, EditorPanelSection } from "@xrf/ui/editor/panel";
import { BaseComponentProps } from "@xrf/ui/element-types";
import { ReactElement } from "react";

import { describeMixedParameters } from "./ArchiveLevelEnvModView.utils";

interface IArchiveLevelEnvModifierSectionProps extends BaseComponentProps {
  modifier: ArchiveLevelEnvModifier;
  /** Position in the file, which is the only name a modifier has. */
  index: number;
}

/**
 * One local weather override: how far it reaches, how strongly it mixes in, and which of the sky's values it touches.
 */
export function ArchiveLevelEnvModifierSection({
  "data-testid": dataTestId = "archive-level-env-modifier-section",
  id,
  className,
  modifier,
  index,
}: IArchiveLevelEnvModifierSectionProps): ReactElement {
  return (
    <EditorPanelSection data-testid={dataTestId} id={id} className={className} title={`Override ${index + 1}`}>
      <EditorPanelProperty
        isInline
        label={"Reaches"}
        value={`${formatNumber(modifier.radius, 1)} m`}
        caption={"Falling off linearly to that edge, so the far side of it changes nothing"}
      />

      <EditorPanelProperty
        isInline
        label={"Mixes in"}
        value={formatNumber(modifier.power, 2)}
        caption={"How much of itself it contributes at the centre"}
      />

      <EditorPanelProperty isInline label={"Far plane"} value={`${formatNumber(modifier.farPlane, 1)} m`} />

      <EditorPanelProperty isInline label={"Fog density"} value={formatNumber(modifier.fogDensity, 3)} />

      <EditorPanelProperty
        isInline
        label={"Touches"}
        value={describeMixedParameters(modifier)}
        caption={
          modifier.declaresParameters
            ? "The values the file says to mix in"
            : "The file carries no flag word, so the engine mixes in all of them"
        }
      />
    </EditorPanelSection>
  );
}
