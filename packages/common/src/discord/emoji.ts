import { z } from "zod";
import { roleSchema } from "./role";
import { userSchema } from "./user";

export const emojiSchema = z.object({
    id: z.string().nullable(),
    name: z.string().nullable(),
    roles: z.array(roleSchema).optional(),
    user: userSchema.optional(),
    require_colons: z.boolean().optional(),
    managed: z.boolean().optional(),
    animated: z.boolean().optional(),
    available: z.boolean().optional(),
});
