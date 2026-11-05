import { EditorPopoverGroupSection } from "@xrf/ui/editor/popover-group";
import { BaseComponentProps } from "@xrf/ui/element-types";
import { ReactElement, ReactNode } from "react";

import { RenderSettingEntries } from "@/core/render/components/settings/RenderSettingEntries";
import { RENDER_CAMERA_SETTINGS } from "@/core/render/lib/features/image/render-camera-choices";
import { describeRenderSetting } from "@/core/render/lib/features/render-setting";
import {
  IRenderFeatureSettings,
  TRenderCameraSettings,
} from "@/core/render/lib/settings/choice/render-feature-settings";

/** A camera effect, each turned on by its own section. */
export type TLevelCameraEffect = "isAberrated" | "isDefocused" | "isGrained" | "isMotionBlurred" | "isVignetted";

export interface ILevelCameraEffectSectionProps extends BaseComponentProps {
  /** The effect the section turns on. */
  effect: TLevelCameraEffect;
  /** The values shaping it, as their entries, top to bottom. */
  names: ReadonlyArray<keyof TRenderCameraSettings>;
  /** The camera as the view draws it. */
  camera: TRenderCameraSettings;
  /** Every feature as the view draws them, which a value's rule reads. */
  features: IRenderFeatureSettings;
  /** Sets part of the view's camera, the rest as it was. */
  onSet: (part: Partial<TRenderCameraSettings>) => void;
  /** What the section holds past its values, such as the focus's pick. */
  children?: ReactNode;
}

/** One camera effect's section of the Camera effects popover, titled and described by the effect's declaration. */
export function LevelCameraEffectSection({
  "data-testid": dataTestId,
  id,
  className,
  effect,
  names,
  camera,
  features,
  onSet,
  children,
}: ILevelCameraEffectSectionProps): ReactElement {
  return (
    <EditorPopoverGroupSection
      data-testid={dataTestId}
      id={id}
      className={className}
      label={RENDER_CAMERA_SETTINGS[effect].label}
      description={describeRenderSetting(RENDER_CAMERA_SETTINGS[effect], camera, features)}
      isOn={camera[effect]}
      onToggle={() => onSet({ [effect]: !camera[effect] })}
    >
      <RenderSettingEntries
        settings={RENDER_CAMERA_SETTINGS}
        names={names}
        group={camera}
        features={features}
        onSet={onSet}
      />

      {children}
    </EditorPopoverGroupSection>
  );
}
