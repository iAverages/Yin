import { z } from "zod";

// TODO: Add all the other types (e.g. team object, partial user object, etc.)
export const applicationSchema = z.object({
    id: z.string(),
    name: z.string(),
    icon: z.string().nullable(),
    description: z.string(),
    rpc_origins: z.array(z.string()).optional(),
    bot_public: z.boolean(),
    bot_require_code_grant: z.boolean(),
    terms_of_service_url: z.string().optional(),
    privacy_policy_url: z.string().optional(),
    owner: z.object({}).optional(),
    verify_key: z.string(),
    team: z.object({}).nullable(),
    guild_id: z.string().optional(),
    primary_sku_id: z.string().optional(),
    slug: z.string().optional(),
    cover_image: z.string().optional(),
    flags: z.number().optional(),
    tags: z.array(z.string()).optional(),
    install_params: z.object({}).optional(),
    custom_install_url: z.string().optional(),
    role_connections_verification_url: z.string().optional(),
});

export type Application = z.infer<typeof applicationSchema>;

export const installParamsSchema = z.object({
    scopes: z.array(z.string()),
    permissions: z.string(),
});

export type InstallParams = z.infer<typeof installParamsSchema>;
