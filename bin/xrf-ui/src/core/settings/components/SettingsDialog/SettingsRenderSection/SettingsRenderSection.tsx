import { Tab, Tabs } from "@mui/material";
import { useInjection } from "@wirestate/react";
import { Nullable } from "@xrf/types";
import { ComponentType, ReactElement, RefObject, useRef, useState } from "react";

import { SettingsService } from "@/core/settings/services/settings";

import {
  ERenderSettingsTab,
  IRenderSettingsTab,
  isCustomRenderSettingsTab,
  isPresetRenderSettingsTab,
  RENDER_SETTINGS_TABS,
} from "./render-settings-tabs";
import { SettingsRendererPreset } from "./SettingsRendererPreset";

/**
 * How every viewport draws, which is neither application chrome nor any one editor's business: in tabs by what a
 * setting changes, the preset over every tab it sets, and a mark on each tab where something differs from it.
 */
export function SettingsRenderSection(): ReactElement {
  const settingsService: SettingsService = useInjection(SettingsService);

  const rootRef: RefObject<Nullable<HTMLDivElement>> = useRef<Nullable<HTMLDivElement>>(null);
  const [tabId, setTabId] = useState<ERenderSettingsTab>(ERenderSettingsTab.DISPLAY);
  const tab: IRenderSettingsTab =
    RENDER_SETTINGS_TABS.find((it: IRenderSettingsTab) => it.id === tabId) ?? RENDER_SETTINGS_TABS[0];

  return (
    <div ref={rootRef} className={"flex scroll-mt-6 flex-col gap-6"}>
      <Tabs
        className={"sticky -top-6 z-1 border-b border-divider surface-content"}
        value={tab.id}
        variant={"scrollable"}
        onChange={(_: unknown, id: ERenderSettingsTab) => {
          setTabId(id);
          // A tab opens where the dialog first showed it, not where the last one was left.
          rootRef.current?.scrollIntoView({ block: "start" });
        }}
      >
        {RENDER_SETTINGS_TABS.map((it: IRenderSettingsTab) => (
          <Tab
            key={it.id}
            value={it.id}
            label={
              <span className={"flex items-center gap-1.5"}>
                {it.label}
                {isCustomRenderSettingsTab(it, settingsService.rendererChoice) ? (
                  <span
                    aria-label={"changed from the preset"}
                    className={"size-1.5 rounded-full bg-primary"}
                    role={"img"}
                  />
                ) : null}
              </span>
            }
          />
        ))}
      </Tabs>

      {isPresetRenderSettingsTab(tab) ? <SettingsRendererPreset /> : null}

      {tab.sections.map((Section: ComponentType, index: number) => (
        <Section key={index} />
      ))}
    </div>
  );
}
