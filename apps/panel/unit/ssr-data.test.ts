import assert from "node:assert/strict";
import { afterEach, test, vi } from "vitest";

import { SSR_DATA_BUDGET_MS, waitForSsrData } from "../src/lib/ssr-data.ts";

const deferred = () => {
    let resolve!: () => void;
    let reject!: (error: Error) => void;
    const promise = new Promise<void>((yes, no) => {
        resolve = yes;
        reject = no;
    });
    return { promise, resolve, reject };
};

afterEach(() => {
    vi.useRealTimers();
});

test("ready data returns immediately and clears its timer", async () => {
    vi.useFakeTimers({ toFake: ["setTimeout", "Date"], now: 1000 });
    const clear = vi.spyOn(globalThis, "clearTimeout");
    await waitForSsrData(Promise.resolve(), Date.now() + SSR_DATA_BUDGET_MS);
    assert.equal(Date.now(), 1000);
    assert.equal(clear.mock.calls.length, 1);
});

test("pending data releases rendering at 2 seconds, not the API request", async () => {
    vi.useFakeTimers({ toFake: ["setTimeout", "Date"], now: 1000 });
    const fetch = deferred();
    let fetched = false;
    void fetch.promise.then(() => {
        fetched = true;
    });
    let rendered = false;
    const wait = waitForSsrData(fetch.promise, Date.now() + SSR_DATA_BUDGET_MS).then(() => {
        rendered = true;
    });
    vi.advanceTimersByTime(SSR_DATA_BUDGET_MS - 1);
    await Promise.resolve();
    assert.equal(rendered, false);
    vi.advanceTimersByTime(1);
    await wait;
    assert.equal(fetched, false);
    fetch.resolve();
    await fetch.promise;
    assert.equal(fetched, true);
});

test("later loaders share the remaining budget instead of adding 2 seconds", async () => {
    vi.useFakeTimers({ toFake: ["setTimeout", "Date"], now: 1000 });
    const deadline = Date.now() + SSR_DATA_BUDGET_MS;
    const directory = deferred();
    const settings = deferred();
    const first = waitForSsrData(directory.promise, deadline);
    vi.advanceTimersByTime(1500);
    const second = waitForSsrData(settings.promise, deadline);
    vi.advanceTimersByTime(500);
    await Promise.all([first, second]);
    assert.equal(Date.now(), deadline);
    directory.resolve();
    settings.resolve();
});

test("an expired deadline does not wait for the request", async () => {
    vi.useFakeTimers({ toFake: ["setTimeout", "Date"], now: 3000 });
    const fetch = deferred();
    const wait = waitForSsrData(fetch.promise, 2000);
    vi.advanceTimersByTime(0);
    await wait;
    assert.equal(Date.now(), 3000);
    fetch.resolve();
});

test("early rejection is handled and clears the timer", async () => {
    vi.useFakeTimers({ toFake: ["setTimeout", "Date"], now: 1000 });
    const clear = vi.spyOn(globalThis, "clearTimeout");
    const error = new Error("API unavailable");
    await assert.rejects(waitForSsrData(Promise.reject(error), 3000), error);
    assert.equal(clear.mock.calls.length, 1);
});

test("late rejection remains handled after rendering has been released", async () => {
    vi.useFakeTimers({ toFake: ["setTimeout", "Date"], now: 1000 });
    const fetch = deferred();
    const wait = waitForSsrData(fetch.promise, 3000);
    vi.advanceTimersByTime(2000);
    await wait;
    // Vitest fails the run if the still-running promise becomes unhandled.
    fetch.reject(new Error("Late API failure"));
    await Promise.resolve();
});
