import { z } from "zod";

export const embedSchema = z.object({
    title: z.string().optional(),
    type: z.string().optional(),
    description: z.string().optional(),
    url: z.string().optional(),
    timestamp: z.string().optional(),
    color: z.number().optional(),
    footer: z
        .object({
            text: z.string().optional(),
            icon_url: z.string().optional(),
            proxy_icon_url: z.string().optional(),
        })
        .optional(),
    image: z
        .object({
            url: z.string(),
            proxy_url: z.string().optional(),
            height: z.number().optional(),
            width: z.number().optional(),
        })
        .optional(),
    thumbnail: z
        .object({
            url: z.string().optional(),
            proxy_url: z.string().optional(),
            height: z.number().optional(),
            width: z.number().optional(),
        })
        .optional(),
    video: z
        .object({
            url: z.string().optional(),
            height: z.number().optional(),
            width: z.number().optional(),
        })
        .optional(),
    provider: z
        .object({
            name: z.string().optional(),
            url: z.string().optional(),
        })
        .optional(),
    author: z
        .object({
            name: z.string(),
            url: z.string().optional(),
            icon_url: z.string().optional(),
            proxy_icon_url: z.string().optional(),
        })
        .optional(),
    fields: z

        .object({
            name: z.string(),
            value: z.string(),
            inline: z.boolean().optional(),
        })
        .array()
        .optional(),
});

export type Embed = z.infer<typeof embedSchema>;
