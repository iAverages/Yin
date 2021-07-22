type JsonValue = string | number | object | Array<any> | boolean | null;

export const getPath = (object: Record<string, any>, path: string | Array<string>): JsonValue => {
    if (typeof path === "string") path = path.split(".");
    const val = object[path[0]];
    if (typeof val === "object") {
        path.shift();
        return getPath(val, path);
    }
    return val;
};

export const setPath = (
    object: Record<string, any>,
    path: string | Array<string>,
    value: JsonValue
): Record<string, JsonValue> => {
    if (typeof path === "string") path = path.split(".");
    const key = path[0];
    if (path.length === 1) {
        object[key] = value;
    } else {
        let newObj = {};
        if (object.hasOwnProperty(key)) newObj = object[key];
        path.shift();
        object[key] = setPath(newObj, path, value);
    }
    return object;
};

export default {
    getPath,
    setPath,
};
