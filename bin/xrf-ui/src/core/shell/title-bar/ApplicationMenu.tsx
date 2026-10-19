import { ButtonBase, Tooltip } from "@mui/material";
import { Nullable } from "@xrf/types";
import { ReactElement, useCallback, useState } from "react";

import { IMenubarMenu, Menubar } from "@/core/shell/menu/components/Menubar";
import { useApplicationMenus } from "@/core/shell/menu/use-application-menus";
import { useMenubarAccessKeys } from "@/core/shell/menu/use-menubar-access-keys";
import { MenuGlyph } from "@/core/shell/title-bar/CaptionGlyph";
import { cn } from "@/core/theme/class-name";

/** What showed the menubar: a click opens File at once, as a click on a menubar does; a key only marks it. */
type TMenubarShownBy = "pointer" | "keyboard";

/**
 * The window's menu in the caption: one button until it is used, then the menubar over the breadcrumb.
 */
export function ApplicationMenu(): ReactElement {
  const menus: Array<IMenubarMenu> = useApplicationMenus();

  const [shownBy, setShownBy] = useState<Nullable<TMenubarShownBy>>(null);

  const onClick = useCallback(() => setShownBy("pointer"), []);

  const onAccessKey = useCallback(() => setShownBy("keyboard"), []);

  const onCollapse = useCallback(() => setShownBy(null), []);

  useMenubarAccessKeys(onAccessKey);

  // The button keeps its room under the open bar, and the bar lies over the breadcrumb, which hides while it is there:
  // opening the menu moves nothing else in the caption. It starts at the rail's edge, after the mark's column, and its
  // margin matches the mark's, so the glyph sits as far from the mark as from the title beside it.
  return (
    <div className={"relative mr-2 flex shrink-0 items-center"}>
      <Tooltip describeChild title={"Menu (Alt)"}>
        <ButtonBase
          aria-haspopup={"menu"}
          aria-label={"Menu"}
          className={cn(
            "size-7 shrink-0 rounded-control text-text-primary hover:bg-action-hover",
            "focus-visible:outline-1 focus-visible:outline-offset-1 focus-visible:outline-primary",
            shownBy ? "invisible" : null
          )}
          disableRipple={true}
          onClick={onClick}
        >
          <MenuGlyph />
        </ButtonBase>
      </Tooltip>

      {shownBy ? (
        <Menubar
          className={"absolute top-0 left-0 z-10"}
          label={"Application menu"}
          menus={menus}
          isFirstMenuOpen={shownBy === "pointer"}
          onCollapse={onCollapse}
        />
      ) : null}
    </div>
  );
}
