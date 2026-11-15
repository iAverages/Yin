import { Link, createFileRoute } from "@tanstack/solid-router";

const linkClass = "text-foreground underline underline-offset-2 hover:text-accent-hover";

const Terms = () => {
    return (
        <main class="mx-auto w-full max-w-[640px] px-5 py-12">
            <article class="leading-relaxed text-muted [&_h2]:mt-10 [&_h2]:mb-3 [&_h2]:text-lg [&_h2]:font-semibold [&_h2]:text-foreground [&_li]:mt-2 [&_p]:my-3 [&_ul]:my-3 [&_ul]:list-disc [&_ul]:pl-5">
                <h1 class="m-0 text-[32px] leading-tight font-semibold tracking-[-.035em] text-foreground">
                    Terms of service
                </h1>
                <p>Last updated 15 November 2026</p>
                <p>
                    These are the rules for using Yin, both the Discord bot and this website. By
                    adding Yin to a server, installing it on your account or signing in here, you
                    agree to them.
                </p>

                <h2>Using Yin</h2>
                <ul>
                    <li>
                        Follow Discord's{" "}
                        <a class={linkClass} href="https://discord.com/terms">
                            Terms of Service
                        </a>{" "}
                        and{" "}
                        <a class={linkClass} href="https://discord.com/guidelines">
                            Community Guidelines
                        </a>
                        .
                    </li>
                    <li>
                        Don't abuse it. No spamming it, no trying to break it or get around its
                        limits, and no using it to harass people.
                    </li>
                    <li>
                        If you add Yin to a server, you're responsible for how it's set up there.
                    </li>
                </ul>

                <h2>Embedded posts</h2>
                <p>
                    Yin shows posts from other sites, like X, Bluesky, Instagram, Facebook, TikTok,
                    Reddit and Spotify. That content belongs to whoever posted it, not to Yin, and
                    Yin isn't affiliated with any of those sites. Embeds can break or go missing
                    when those sites change.
                </p>

                <h2>No guarantees</h2>
                <p>
                    Yin is a free hobby project and is provided as is. It might be down, have bugs,
                    lose settings or shut down for good. As far as the law allows, I'm not liable
                    for any loss or damage that comes from using it.
                </p>

                <h2>Losing access</h2>
                <p>
                    I can block a user or a server from Yin at any time, for example for breaking
                    these terms.
                </p>

                <h2>Your data</h2>
                <p>
                    What Yin keeps about you is covered in the{" "}
                    <Link to="/privacy" class={linkClass}>
                        privacy policy
                    </Link>
                    .
                </p>

                <h2>Changes</h2>
                <p>
                    If these terms change, the date at the top changes. Older versions are in the
                    project's history on GitHub.
                </p>

                <h2>Contact</h2>
                <p>
                    Open an issue at{" "}
                    <a class={linkClass} href="https://github.com/iAverages/Yin/issues">
                        github.com/iAverages/Yin
                    </a>
                    .
                </p>
            </article>
        </main>
    );
};

export const Route = createFileRoute("/terms")({
    component: Terms,
    head: () => ({
        meta: [
            { title: "Terms of service | Yin" },
            { name: "description", content: "The rules for using Yin." },
        ],
    }),
});
