import { onCleanup } from "solid-js";

export const onSidebarShortcut = (desktop: boolean, toggle: () => void) => {
    const media = window.matchMedia("(min-width: 768px)");
    const handleKeydown = (event: KeyboardEvent) => {
        if (
            media.matches !== desktop ||
            event.defaultPrevented ||
            event.isComposing ||
            !event.ctrlKey ||
            event.altKey ||
            event.metaKey ||
            event.shiftKey ||
            event.key.toLowerCase() !== "b"
        )
            return;
        event.preventDefault();
        if (!event.repeat) toggle();
    };
    window.addEventListener("keydown", handleKeydown);
    onCleanup(() => window.removeEventListener("keydown", handleKeydown));
};
