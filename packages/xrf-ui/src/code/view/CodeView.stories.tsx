import type { Meta, StoryObj } from "@storybook/react-vite";
import { ESyntaxLanguage } from "@xrf/syntax";

import { withReadingCard } from "#/testing/stories";

import { CodeView } from "./CodeView";

const LTX_SOURCE: string = `[wpn_ak74]:identity_immunities,weapon_probability,default_weapon_params
cost                    = 7000 ; base trade price
ammo_class              = ammo_5.45x39_fmj, ammo_5.45x39_ap
ammo_mag_size           = 30
fire_modes              = 1, 3, -1
hit_power               = 0.43, 0.43, 0.43, 0.43
rpm                     = 600
visual                  = dynamics\\weapons\\wpn_ak74\\wpn_ak74.ogf
#include "weapon_upgrades.ltx"
`;

const LUA_SOURCE: string = `function on_actor_hit(actor, amount, direction, who, bone)
\tif amount <= 0 then
\t\treturn
\tend

\tlocal health = actor.health
\tprintf("actor hit by %s for %.2f", who and who:name() or "nil", amount)
end
`;

const meta = {
  title: "Components/Code/CodeView",
  component: CodeView,
  decorators: [withReadingCard],
  args: {
    content: LTX_SOURCE,
    language: ESyntaxLanguage.LTX,
    label: "weapons\\w_ak74.ltx",
  },
} satisfies Meta<typeof CodeView>;

export default meta;

type TStory = StoryObj<typeof meta>;

export const Ltx: TStory = {};

export const Lua: TStory = {
  args: { content: LUA_SOURCE, language: ESyntaxLanguage.LUA, label: "scripts\\bind_actor.script" },
};

/** An excerpt keeps the line numbers of the file it was lifted from. */
export const Excerpt: TStory = {
  args: { firstLine: 412 },
};
