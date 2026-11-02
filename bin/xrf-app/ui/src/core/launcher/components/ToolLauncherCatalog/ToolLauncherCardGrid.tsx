import { BaseComponentProps } from "@xrf/ui/element-types";
import { cn } from "@xrf/ui/theme";
import { ReactElement } from "react";

import { ToolLauncherCard } from "@/core/launcher/components/ToolLauncherCard";
import { ICatalogEntry, ICatalogSection } from "@/core/launcher/lib";
import { IToolDescriptor } from "@/core/routing/tool";

import { ToolLauncherGroupHeading } from "./ToolLauncherGroupHeading";

interface IToolLauncherCardGridProps extends BaseComponentProps {
  sections: ReadonlyArray<ICatalogSection>;
  onOpen: (tool: IToolDescriptor) => void;
}

/**
 * The roomy body: one grid of cards per section, under the heading the section carries.
 */
export function ToolLauncherCardGrid({
  "data-testid": dataTestId = "tool-launcher-card-grid",
  id,
  className,
  sections,
  onOpen,
}: IToolLauncherCardGridProps): ReactElement {
  return (
    <div data-testid={dataTestId} id={id} className={cn("flex flex-col gap-10", className)}>
      {sections.map((section: ICatalogSection) => (
        <div key={section.group?.id ?? "ranked"} className={"flex flex-col gap-3"}>
          {section.group ? <ToolLauncherGroupHeading group={section.group} /> : null}

          <div className={"grid grid-cols-[repeat(auto-fill,minmax(16rem,1fr))] gap-4"}>
            {section.entries.map(({ tool, group }: ICatalogEntry) => (
              <ToolLauncherCard
                key={tool.id}
                tool={tool}
                group={group}
                isGroupNamed={section.group === null}
                onOpen={onOpen}
              />
            ))}
          </div>
        </div>
      ))}
    </div>
  );
}
