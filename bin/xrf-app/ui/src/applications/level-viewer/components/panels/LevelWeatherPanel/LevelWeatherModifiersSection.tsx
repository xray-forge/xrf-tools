import { Typography } from "@mui/material";
import { ReactElement } from "react";

import { EditorPanelProperty, EditorPanelSection } from "@/core/shell/editor/EditorPanel";
import { BaseComponentProps } from "@/lib/dom/element-types";

interface ILevelWeatherModifiersSectionProps extends BaseComponentProps {
  /** The level's `level.env_mod` volumes. */
  count: number;
  /** How many of them reach the camera now. */
  reaching: number;
}

/**
 * The level's own weather overrides, which bend the fog, the far plane and the colours where the camera is inside one.
 */
export function LevelWeatherModifiersSection({
  "data-testid": dataTestId = "level-weather-modifiers-section",
  id,
  className,
  count,
  reaching,
}: ILevelWeatherModifiersSectionProps): ReactElement {
  return (
    <EditorPanelSection data-testid={dataTestId} id={id} className={className} title={"Level overrides"}>
      {count ? (
        <>
          <EditorPanelProperty label={"Volumes"} value={count} />
          <EditorPanelProperty label={"Around the camera"} value={reaching} />
        </>
      ) : (
        <Typography className={"block text-text-secondary"} variant={"caption"}>
          The level has no level.env_mod.
        </Typography>
      )}
    </EditorPanelSection>
  );
}
