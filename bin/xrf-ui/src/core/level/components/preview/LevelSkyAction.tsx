import { default as NightsStayIcon } from "@mui/icons-material/NightsStay";
import { ReactElement } from "react";

import { LevelWeatherTexture } from "@/core/ipc/types/xrf-app";
import { LevelManualWeatherSlider } from "@/core/level/components/weather/LevelManualWeatherSlider";
import { LevelManualWeatherVectorField } from "@/core/level/components/weather/LevelManualWeatherVectorField";
import { LevelWeatherResetButton } from "@/core/level/components/weather/LevelWeatherResetButton";
import { LevelWeatherTextureField } from "@/core/level/components/weather/LevelWeatherTextureField";
import { ILevelViewOptions } from "@/core/level/lib/view/level-view-options";
import { ILevelManualWeather } from "@/core/level/lib/weather/level-manual-weather";
import { EditorPopoverGroup, EditorPopoverGroupSection } from "@/core/shell/editor/EditorPopoverGroup";
import { BaseComponentProps } from "@/lib/dom/element-types";
import { formatDegrees } from "@/lib/format/angle";
import { formatPercent } from "@/lib/format/number";

/** The keys the popover sets. */
const SKY_KEYS: ReadonlyArray<keyof ILevelManualWeather> = [
  "skyTexture",
  "skyColor",
  "skyRotation",
  "cloudsTexture",
  "cloudsColor",
  "cloudsRotation",
];

interface ILevelSkyActionProps extends BaseComponentProps {
  options: ILevelViewOptions;
  /** The keyframe on screen: the one set by hand, or the weather's mix, which the first edit seeds it from. */
  manual: ILevelManualWeather;
  /** Every sky the game's weather names. */
  skies: ReadonlyArray<LevelWeatherTexture>;
  /** Every clouds texture it names. */
  clouds: ReadonlyArray<LevelWeatherTexture>;
  onToggle: (option: keyof ILevelViewOptions) => void;
  onEdit: (patch: Partial<ILevelManualWeather>) => void;
}

/**
 * The sky cube, its tint and turn, which its `#small` twin follows, and the clouds over it with their cover.
 */
export function LevelSkyAction({
  "data-testid": dataTestId = "level-sky-action",
  id,
  className,
  options,
  manual,
  skies,
  clouds,
  onToggle,
  onEdit,
}: ILevelSkyActionProps): ReactElement {
  const { isSkyVisible, isClouded } = options;
  const isCloudy: boolean = isClouded && manual.cloudsTexture !== "";

  return (
    <EditorPopoverGroup
      data-testid={dataTestId}
      id={id}
      className={className}
      label={"Sky"}
      description={[
        isSkyVisible ? `Sky ${manual.skyTexture || "none"}` : "Sky off, the backdrop behind the level",
        isCloudy ? `clouds ${manual.cloudsTexture}, ${formatPercent(manual.cloudsColor[3])} cover` : "no clouds",
      ].join(", ")}
      icon={<NightsStayIcon />}
      isActive={isSkyVisible || isCloudy}
    >
      <EditorPopoverGroupSection label={"Sky"} isOn={isSkyVisible} onToggle={() => onToggle("isSkyVisible")}>
        <LevelWeatherTextureField
          label={"sky_texture"}
          value={manual.skyTexture}
          textures={skies}
          onChange={(skyTexture: string) => onEdit({ skyTexture })}
        />

        <LevelManualWeatherVectorField field={"skyColor"} isColor manual={manual} onEdit={onEdit} />

        <LevelManualWeatherSlider field={"skyRotation"} manual={manual} format={formatDegrees} onEdit={onEdit} />
      </EditorPopoverGroupSection>

      <EditorPopoverGroupSection label={"Clouds"} isOn={isClouded} onToggle={() => onToggle("isClouded")}>
        <LevelWeatherTextureField
          label={"clouds_texture"}
          value={manual.cloudsTexture}
          textures={clouds}
          onChange={(cloudsTexture: string) => onEdit({ cloudsTexture })}
        />

        <LevelManualWeatherVectorField field={"cloudsColor"} isColor manual={manual} onEdit={onEdit} />

        <LevelManualWeatherSlider field={"cloudsRotation"} manual={manual} format={formatDegrees} onEdit={onEdit} />
      </EditorPopoverGroupSection>

      <LevelWeatherResetButton keys={SKY_KEYS} onEdit={onEdit} />
    </EditorPopoverGroup>
  );
}
