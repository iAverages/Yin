import { z } from "zod";

export const applicationRoleConnectionMetadataSchema = z.object({
    type: z.unknown(),
    key: z.string(),
    name: z.string(),
    name_localizations: z.unknown().optional(),
    description: z.string(),
    description_localizations: z.unknown().optional(),
});

export type ApplicationRoleConnectionMetadata = z.infer<typeof applicationRoleConnectionMetadataSchema>;
