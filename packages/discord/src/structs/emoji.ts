import { z } from "zod";

import { user } from "./index";
import { roleSchema } from "./role";

export const emojiSchema = z.object({
    id: z.string().nullable(),
    name: z.string().nullable(),
    roles: z.array(roleSchema).optional(),
    user: user.userSchema.optional(),
    require_colons: z.boolean().optional(),
    managed: z.boolean().optional(),
    animated: z.boolean().optional(),
    available: z.boolean().optional(),
});
