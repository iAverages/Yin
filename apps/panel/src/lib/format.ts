const units = [
    ["days", 86_400],
    ["hours", 3_600],
    ["minutes", 60],
    ["seconds", 1],
] as const;

// A fixed locale keeps server and client output identical for hydration.
export const formatDuration = (totalSeconds: number, style: "long" | "narrow" = "long") => {
    const duration: Partial<Record<(typeof units)[number][0], number>> = {};
    let remaining = totalSeconds;
    for (const [unit, size] of units) {
        duration[unit] = Math.floor(remaining / size);
        remaining %= size;
    }
    return new Intl.DurationFormat("en", {
        style,
        secondsDisplay: totalSeconds === 0 ? "always" : "auto",
    }).format(duration);
};
