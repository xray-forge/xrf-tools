import { EditorPanelProperty, EditorPanelSection } from "@xrf/ui/editor/panel";
import { BaseComponentProps } from "@xrf/ui/element-types";
import { Typography } from "@xrf/ui/material/typography";
import { ReactElement } from "react";

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
