export type Language = { name: string; code: string };

// crates/feature-social/src/lib.rs
const LANGUAGES: [name: string, code: string][] = [
    ["English", "en"],
    ["Japanese", "ja"],
    ["Korean", "ko"],
    ["Chinese (Simplified)", "zh-cn"],
    ["Chinese (Traditional)", "zh-tw"],
    ["Spanish", "es"],
    ["Portuguese", "pt"],
    ["French", "fr"],
    ["German", "de"],
    ["Italian", "it"],
    ["Russian", "ru"],
    ["Ukrainian", "uk"],
    ["Polish", "pl"],
    ["Dutch", "nl"],
    ["Turkish", "tr"],
    ["Arabic", "ar"],
    ["Hindi", "hi"],
    ["Indonesian", "id"],
    ["Thai", "th"],
    ["Vietnamese", "vi"],
    ["Filipino", "tl"],
    ["Swedish", "sv"],
    ["Czech", "cs"],
    ["Greek", "el"],
    ["Hebrew", "he"],
    ["Amharic", "am"],
    ["Armenian", "hy"],
    ["Basque", "eu"],
    ["Bengali", "bn"],
    ["Bulgarian", "bg"],
    ["Catalan", "ca"],
    ["Croatian", "hr"],
    ["Danish", "da"],
    ["Estonian", "et"],
    ["Finnish", "fi"],
    ["Georgian", "ka"],
    ["Gujarati", "gu"],
    ["Hungarian", "hu"],
    ["Kannada", "kn"],
    ["Latvian", "lv"],
    ["Lithuanian", "lt"],
    ["Malayalam", "ml"],
    ["Marathi", "mr"],
    ["Norwegian", "no"],
    ["Persian", "fa"],
    ["Romanian", "ro"],
    ["Serbian", "sr"],
    ["Slovak", "sk"],
    ["Slovenian", "sl"],
    ["Tamil", "ta"],
    ["Telugu", "te"],
    ["Urdu", "ur"],
];

export const languages: Language[] = LANGUAGES.map(([name, code]) => ({ name, code }));

export const matchesLanguage = (language: Language, search: string) => {
    const term = search.trim().toLowerCase();
    return language.name.toLowerCase().includes(term) || language.code.startsWith(term);
};
