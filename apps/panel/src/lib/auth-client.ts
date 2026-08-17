import { createAuthClient } from "better-auth/solid";

import { env } from "../env";

export const authClient = createAuthClient({
    baseURL: env.VITE_AUTH_URL,
});
