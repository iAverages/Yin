import { z } from "zod";
import { user, role } from "./index";

export const emojiSchema = z.object({
    id: z.string().nullable(),
    name: z.string().nullable(),
    roles: z.array(role.roleSchema).optional(),
    user: user.userSchema.optional(),
    require_colons: z.boolean().optional(),
    managed: z.boolean().optional(),
    animated: z.boolean().optional(),
    available: z.boolean().optional(),
});
