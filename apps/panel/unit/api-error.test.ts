import assert from "node:assert/strict";
import { test } from "vitest";

import { ApiRequestError, isRetryableError } from "../src/lib/api-error.ts";

test("client errors are not retried", () => {
    for (const status of [400, 401, 403, 404, 409, 422]) {
        assert.equal(isRetryableError(new ApiRequestError(status, "")), false, `status ${status}`);
    }
});

test("server, timeout, rate limit and network errors are retried", () => {
    for (const status of [408, 429, 500, 503]) {
        assert.equal(isRetryableError(new ApiRequestError(status, "")), true, `status ${status}`);
    }
    assert.equal(isRetryableError(new TypeError("fetch failed")), true);
});
