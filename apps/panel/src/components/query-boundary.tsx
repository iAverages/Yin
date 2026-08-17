import { useQueryClient } from "@tanstack/solid-query";
import { ErrorBoundary, Suspense, type JSX } from "solid-js";

import { Button } from "./button";

export const QueryError = (props: { error: unknown; onRetry: () => void; pending?: boolean }) => {
    return (
        <div class="rounded-lg border border-danger/25 bg-danger/[.08] p-5" role="alert">
            <p class="m-0 text-sm text-[#ff99a1]">
                {props.error instanceof Error ? props.error.message : "Could not load this data."}
            </p>
            <Button
                class="mt-3"
                size="small"
                variant="ghost"
                disabled={props.pending}
                onClick={props.onRetry}
            >
                Try again
            </Button>
        </div>
    );
};

export const QueryBoundary = (props: {
    queryKey: readonly unknown[];
    fallback: JSX.Element;
    children: JSX.Element;
}) => {
    const queryClient = useQueryClient();
    return (
        <ErrorBoundary
            fallback={(error, reset) => (
                <QueryError
                    error={error}
                    onRetry={() => {
                        void queryClient.resetQueries({ queryKey: props.queryKey });
                        reset();
                    }}
                />
            )}
        >
            <Suspense fallback={props.fallback}>{props.children}</Suspense>
        </ErrorBoundary>
    );
};
