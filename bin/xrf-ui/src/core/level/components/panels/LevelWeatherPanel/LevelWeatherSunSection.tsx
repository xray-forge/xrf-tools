import { Typography } from "@mui/material";
import { ReactElement } from "react";

import { EXrayEngine, XrayEngine } from "@/core/ipc/types/xrf-engine-target";
import { EditorPanelSection } from "@/core/shell/editor/EditorPanel";
import { CheckboxFormRow } from "@/core/ui/form/CheckboxFormRow";
import { BaseComponentProps } from "@/lib/dom/element-types";

interface ILevelWeatherSunSectionProps extends BaseComponentProps {
  engine: XrayEngine;
  isDynamicSun: boolean;
  isDisabled?: boolean;
  onChange: (isDynamicSun: boolean) => void;
}

/**
 * Where the sun stands: the keyframes' angles or OpenXRay's astronomical sun on vanilla, the sun table on extended.
 */
export function LevelWeatherSunSection({
  "data-testid": dataTestId = "level-weather-sun-section",
  id,
  className,
  engine,
  isDynamicSun,
  isDisabled = false,
  onChange,
}: ILevelWeatherSunSectionProps): ReactElement {
  return (
    <EditorPanelSection data-testid={dataTestId} id={id} className={className} title={"Sun"}>
      {engine === EXrayEngine.VANILLA ? (
        <CheckboxFormRow
          label={"Dynamic sun"}
          description={
            isDynamicSun
              ? "Computed for the time of day, as OpenXRay does by default"
              : "At the angles the keyframes write"
          }
          isChecked={isDynamicSun}
          isDisabled={isDisabled}
          onChange={onChange}
        />
      ) : (
        <Typography className={"block text-text-secondary"} variant={"caption"}>
          Stood by the game&apos;s sun table, hour by hour.
        </Typography>
      )}
    </EditorPanelSection>
  );
}
