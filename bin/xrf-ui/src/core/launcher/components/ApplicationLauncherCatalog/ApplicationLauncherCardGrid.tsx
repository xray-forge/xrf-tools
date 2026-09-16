import { ReactElement } from "react";

import { ApplicationLauncherCard } from "@/core/launcher/components/ApplicationLauncherCard";
import { ICatalogEntry, ICatalogSection } from "@/core/launcher/lib";
import { IApplicationDescriptor } from "@/core/routing/application";
import { cn } from "@/lib/dom/dom-name";
import { BaseComponentProps } from "@/lib/dom/element-types";

import { ApplicationLauncherGroupHeading } from "./ApplicationLauncherGroupHeading";

interface IApplicationLauncherCardGridProps extends BaseComponentProps {
  sections: ReadonlyArray<ICatalogSection>;
  onOpen: (application: IApplicationDescriptor) => void;
}

/**
 * The roomy body: one grid of cards per section, under the heading the section carries.
 */
export function ApplicationLauncherCardGrid({
  "data-testid": dataTestId = "application-launcher-card-grid",
  id,
  className,
  sections,
  onOpen,
}: IApplicationLauncherCardGridProps): ReactElement {
  return (
    <div data-testid={dataTestId} id={id} className={cn("flex flex-col gap-8", className)}>
      {sections.map((section: ICatalogSection) => (
        <div key={section.group?.id ?? "ranked"} className={"flex flex-col gap-2"}>
          {section.group ? (
            <ApplicationLauncherGroupHeading group={section.group} count={section.entries.length} />
          ) : null}

          <div className={"grid grid-cols-1 gap-3 sm:grid-cols-2 md:grid-cols-3 xl:grid-cols-4"}>
            {section.entries.map(({ application, group }: ICatalogEntry) => (
              <ApplicationLauncherCard
                key={application.id}
                application={application}
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
