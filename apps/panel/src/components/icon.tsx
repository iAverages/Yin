import arrowRightIcon from "../assets/icons/arrow-right.svg";
import plusIcon from "../assets/icons/plus.svg";
import checkIcon from "../assets/icons/check.svg";
import chevronsIcon from "../assets/icons/chevrons-up-down.svg";
import panelLeftIcon from "../assets/icons/panel-left.svg";
import logOutIcon from "../assets/icons/log-out.svg";
import chevronIcon from "../assets/icons/chevron-down.svg";
import discordIcon from "../assets/icons/discord.svg";
import overviewIcon from "../assets/icons/layout-dashboard.svg";
import searchIcon from "../assets/icons/search.svg";
import gearIcon from "../assets/icons/settings.svg";
import moderationIcon from "../assets/icons/shield-check.svg";
import closeIcon from "../assets/icons/x.svg";

export type IconName =
    | "arrow-right"
    | "plus"
    | "check"
    | "chevrons"
    | "panel-left"
    | "log-out"
    | "chevron"
    | "close"
    | "discord"
    | "gear"
    | "moderation"
    | "overview"
    | "search";

const icons: Record<IconName, string> = {
    "arrow-right": arrowRightIcon,
    plus: plusIcon,
    check: checkIcon,
    chevrons: chevronsIcon,
    "panel-left": panelLeftIcon,
    "log-out": logOutIcon,
    chevron: chevronIcon,
    close: closeIcon,
    discord: discordIcon,
    gear: gearIcon,
    moderation: moderationIcon,
    overview: overviewIcon,
    search: searchIcon,
};

export const Icon = (props: { name: IconName; size?: number }) => {
    const size = () => `${props.size ?? 18}px`;
    const style = () => {
        const mask = `url("${icons[props.name]}") center / contain no-repeat`;
        return `width:${size()};height:${size()};-webkit-mask:${mask};mask:${mask}`;
    };

    return (
        <span
            class="inline-block shrink-0 bg-current align-middle"
            style={style()}
            aria-hidden="true"
        />
    );
};
