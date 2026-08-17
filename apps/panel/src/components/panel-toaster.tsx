import { ClientOnly } from "@tanstack/solid-router";
import { Toaster } from "solid-sonner";

export const PanelToaster = () => {
    return (
        <ClientOnly>
            <Toaster
                theme="dark"
                position="bottom-right"
                closeButton
                duration={5000}
                mobileOffset={{
                    bottom: "max(16px, env(safe-area-inset-bottom))",
                    left: 16,
                    right: 16,
                }}
                toastOptions={{
                    closeButtonAriaLabel: "Dismiss notification",
                    classNames: {
                        toast: "!rounded-lg !border-line !bg-elevated !font-sans !text-foreground !shadow-lg",
                        title: "text-sm font-medium",
                        description: "!text-xs !text-muted",
                        success: "[&_[data-icon]]:text-success",
                        error: "[&_[data-icon]]:text-danger",
                    },
                }}
            />
        </ClientOnly>
    );
};
