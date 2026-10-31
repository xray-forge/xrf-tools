import { LevelObjectKind, LevelObjectKindEntry } from "@xrf/ipc/types/xrf-app-levels";
import { ERenderHelperIcon, RenderHelperIcon } from "@xrf/ipc/types/xrf-renderer";
import { Nullable } from "@xrf/types";
import { BaseComponentProps } from "@xrf/ui/element-types";
import { AutoAwesomeIcon } from "@xrf/ui/icons/auto-awesome";
import { FlagIcon } from "@xrf/ui/icons/flag";
import { GroupsIcon } from "@xrf/ui/icons/groups";
import { HubIcon } from "@xrf/ui/icons/hub";
import { LightbulbIcon } from "@xrf/ui/icons/lightbulb";
import { PersonIcon } from "@xrf/ui/icons/person";
import { PetsIcon } from "@xrf/ui/icons/pets";
import { PlaceIcon } from "@xrf/ui/icons/place";
import { RestartAltIcon } from "@xrf/ui/icons/restart-alt";
import { ShieldIcon } from "@xrf/ui/icons/shield";
import { StorefrontIcon } from "@xrf/ui/icons/storefront";
import { VolumeUpIcon } from "@xrf/ui/icons/volume-up";
import { cn } from "@xrf/ui/theme";
import { ReactElement } from "react";

import { getLevelObjectKind } from "@/core/level/lib/objects/level-object-kinds";

/**
 * The Material icon each helper icon is rasterised from into the renderer's atlas, as `RENDER_HELPER_ICONS` names it,
 * so a kind's row shows the picture its helpers draw.
 */
export const LEVEL_KIND_ICONS: Readonly<Record<RenderHelperIcon, typeof LightbulbIcon>> = {
  [ERenderHelperIcon.LIGHT]: LightbulbIcon,
  [ERenderHelperIcon.SOUND]: VolumeUpIcon,
  [ERenderHelperIcon.PARTICLES]: AutoAwesomeIcon,
  [ERenderHelperIcon.ACTOR_START]: FlagIcon,
  [ERenderHelperIcon.STALKER]: PersonIcon,
  [ERenderHelperIcon.CREATURE]: PetsIcon,
  [ERenderHelperIcon.TRADER]: StorefrontIcon,
  [ERenderHelperIcon.SQUAD]: GroupsIcon,
  [ERenderHelperIcon.SMART_COVER]: ShieldIcon,
  [ERenderHelperIcon.PATROL_POINT]: PlaceIcon,
  [ERenderHelperIcon.GRAPH_POINT]: HubIcon,
  [ERenderHelperIcon.RESPAWN_POINT]: RestartAltIcon,
};

interface ILevelKindIconProps extends BaseComponentProps {
  kind: LevelObjectKind;
}

/**
 * The icon a kind's helpers draw where its objects stand, in its colour; nothing for a kind drawing none.
 */
export function LevelKindIcon({
  "data-testid": dataTestId = "level-kind-icon",
  id,
  className,
  kind,
}: ILevelKindIconProps): Nullable<ReactElement> {
  const entry: LevelObjectKindEntry = getLevelObjectKind(kind);

  if (!entry.icon) {
    return null;
  }

  const Icon: typeof LightbulbIcon = LEVEL_KIND_ICONS[entry.icon];

  return (
    <Icon
      data-testid={dataTestId}
      id={id}
      className={cn("shrink-0", className)}
      sx={{ color: entry.color, fontSize: 14 }}
      aria-hidden
    />
  );
}
