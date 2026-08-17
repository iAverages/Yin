import assert from "node:assert/strict";
import { test } from "vitest";

import { formatDuration } from "../src/lib/format.ts";

test("long durations name each non-zero unit", () => {
    assert.equal(formatDuration(2_592_000), "30 days");
    assert.equal(formatDuration(90_000), "1 day, 1 hour");
});

test("narrow durations abbreviate units", () => {
    assert.equal(formatDuration(7_500, "narrow"), "2h 5m");
});

test("a zero duration still renders a unit", () => {
    assert.equal(formatDuration(0, "narrow"), "0s");
});
