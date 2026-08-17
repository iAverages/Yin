export const SSR_DATA_BUDGET_MS = 2_000;

export const waitForSsrData = async (
    prefetch: Promise<unknown>,
    deadline: number,
): Promise<void> => {
    let timer: ReturnType<typeof setTimeout> | undefined;
    try {
        await Promise.race([
            prefetch,
            new Promise<void>((resolve) => {
                timer = setTimeout(resolve, Math.max(0, deadline - Date.now()));
            }),
        ]);
    } finally {
        clearTimeout(timer);
    }
};
