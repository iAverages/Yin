export const isNoneEmptyArray = <T>(val: T[] | undefined | null): val is T[] => isDefined(val) && val.length > 0;

export const isDefined = <T>(val: T | undefined | null): val is T => val !== undefined && val !== null;
