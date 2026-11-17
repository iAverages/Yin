import assert from "node:assert/strict";
import { test } from "vitest";

import { languages, matchesLanguage } from "../src/lib/languages.ts";

const search = (term: string) =>
    languages.filter((language) => matchesLanguage(language, term)).map(({ code }) => code);

test("languages are found by name or by the start of their code", () => {
    assert.deepEqual(search(" JAPAN "), ["ja"]);
    assert.deepEqual(search("zh"), ["zh-cn", "zh-tw"]);
    assert.deepEqual(search("pt-PT"), []);
    assert.equal(search("").length, languages.length);
});
