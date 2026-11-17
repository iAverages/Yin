import { createFileRoute } from "@tanstack/solid-router";

const linkClass = "text-foreground underline underline-offset-2 hover:text-accent-hover";

const Privacy = () => {
    return (
        <main class="mx-auto w-full max-w-[640px] px-5 py-12">
            <article class="leading-relaxed text-muted [&_h2]:mt-10 [&_h2]:mb-3 [&_h2]:text-lg [&_h2]:font-semibold [&_h2]:text-foreground [&_li]:mt-2 [&_p]:my-3 [&_strong]:font-medium [&_strong]:text-foreground [&_ul]:my-3 [&_ul]:list-disc [&_ul]:pl-5">
                <h1 class="m-0 text-[32px] leading-tight font-semibold tracking-[-.035em] text-foreground">
                    Privacy policy
                </h1>
                <p>Last updated 15 November 2026</p>
                <p>
                    Yin is a Discord bot with a web panel on this site. It's a hobby project run by
                    one person, iAverages. This page covers what Yin keeps about you, why, and how
                    to get it removed. It applies to both the bot and the website.
                </p>

                <h2>What the bot keeps</h2>
                <p>
                    <strong>Server settings.</strong> The server's ID, name and icon, its command
                    prefix, translation language, which platforms have embeds turned off and custom
                    commands.
                </p>
                <p>
                    <strong>Your embed settings.</strong> Your Discord user ID, the platforms you've
                    turned embeds off for, how you want embeds shown, and your default translation
                    language if you set one. The first time Yin embeds one of your links it also
                    saves your user ID, so the intro notice only shows once.
                </p>
                <p>
                    <strong>Error logs.</strong> If a command fails, the log can include your user
                    ID, the server and channel ID, and the command name. Logs are only used to fix
                    bugs.
                </p>

                <h2>Messages</h2>
                <p>
                    Yin can read messages in the servers it's in. It checks each one for two things:
                    links it can embed, and commands. It can't read DMs. In a DM it only sees the
                    slash commands you run. Apart from what you give a command (like a custom
                    command's reply), message text isn't saved to the database or written to logs.
                    If Yin embedded a link from your message, the text is held in memory for up to
                    five minutes so the embed can be updated or removed when you edit or delete the
                    message.
                </p>

                <h2>What the website keeps</h2>
                <p>You sign in with Discord. When you do, the site saves:</p>
                <ul>
                    <li>your Discord user ID, display name, avatar and email address</li>
                    <li>
                        the access tokens Discord issues, which are used to read your server list
                    </li>
                    <li>
                        a copy of your server list (names, icons and your permissions in each,
                        including servers Yin isn't in), so the panel knows which ones you can
                        manage
                    </li>
                    <li>a session record with your IP address and browser user agent</li>
                </ul>
                <p>
                    Your email is only shown back to you in the account menu. Yin doesn't send
                    email. The site sets a cookie to keep you signed in. There are no ads on it.
                </p>
                <p>
                    The site uses PostHog for analytics and error reports. It records which pages
                    you open, your browser and device, your IP address, and details of any errors
                    you run into. PostHog keeps an ID in your browser to tell visits apart.
                </p>

                <h2>Who else sees data</h2>
                <ul>
                    <li>
                        <strong>Discord.</strong> Yin runs on Discord, so everything it does goes
                        through them. Avatars and server icons on the website load from Discord.
                    </li>
                    <li>
                        <strong>Embed services.</strong> To build an embed, Yin asks another service
                        for the post: FxEmbed or abembed.com. They get the link and sometimes a
                        language code. They aren't told who posted it or where.
                    </li>
                    <li>
                        <strong>PostHog</strong> (hosted in the EU). It handles the website's
                        analytics and error reports described above. The bot also uses it for
                        feature flags, which turn a feature on for some users or servers and not
                        others. To check a flag, the bot sends PostHog your Discord user ID and the
                        ID of the server you're in. The bot also sends PostHog the ID and name of
                        every server Yin is in.
                    </li>
                </ul>
                <p>Nothing is sold, used for ads, or used to train AI.</p>

                <h2>How long it's kept</h2>
                <p>
                    Settings stay until they're changed or deleted. When Yin is removed from a
                    server, that server's data is kept for 30 days in case it's added back, then
                    deleted. Website sign-ins expire a week after you last used the site. Your
                    website account stays until you ask for it to be deleted.
                </p>

                <h2>Your options</h2>
                <ul>
                    <li>
                        Turn embeds off for any platform with <code>/settings</code> in Discord, or
                        under Settings on the website. Server admins can turn them off for a whole
                        server.
                    </li>
                    <li>
                        To cut off the website's access to your Discord account, remove Yin under
                        User Settings &gt; Authorized Apps in Discord.
                    </li>
                    <li>
                        To get a copy of your data or have it deleted, contact me with your Discord
                        user ID. Server owners can ask for everything tied to their server to be
                        deleted.
                    </li>
                </ul>

                <h2>Age</h2>
                <p>
                    Discord requires its users to be at least 13, or older in some countries. Yin
                    isn't meant for anyone younger than that.
                </p>

                <h2>Changes</h2>
                <p>
                    If this policy changes, the date at the top changes. Older versions are in the
                    project's history on GitHub.
                </p>

                <h2>Contact</h2>
                <p>
                    Open an issue at{" "}
                    <a class={linkClass} href="https://github.com/iAverages/Yin/issues">
                        github.com/iAverages/Yin
                    </a>
                    . Issues are public, so don't post anything private there. Your Discord user ID
                    is enough.
                </p>
            </article>
        </main>
    );
};

export const Route = createFileRoute("/privacy")({
    component: Privacy,
    head: () => ({
        meta: [
            { title: "Privacy policy | Yin" },
            { name: "description", content: "What Yin keeps about you and how to remove it." },
        ],
    }),
});
