import {
    Link,
    Outlet,
    createFileRoute,
    notFound,
    redirect,
    useRouter,
} from "@tanstack/solid-router";

import { QueryError } from "../components/query-boundary";
import { getAdminAccess } from "../lib/admin-queries";
import { ApiRequestError } from "../lib/api-error";
import { loginRedirect } from "../lib/auth.functions";
import { SSR_DATA_BUDGET_MS } from "../lib/ssr-data";

const AdminRouteError = (props: { error: Error }) => {
    const router = useRouter();
    return <QueryError error={props.error} onRetry={() => void router.invalidate()} />;
};

export const Route = createFileRoute("/_dashboard/admin")({
    context: ({ context }) => ({ breadcrumbs: [...context.breadcrumbs, { label: "Admin" }] }),
    beforeLoad: async ({ location }) => {
        try {
            if (!(await getAdminAccess())) throw notFound();
        } catch (error) {
            if (error instanceof ApiRequestError) {
                if (error.status === 401)
                    throw redirect({ to: "/login", search: loginRedirect(location.href) });
                if (error.status === 403 || error.status === 404) throw notFound();
            }
            throw error;
        }
        return { ssrDataDeadline: import.meta.env.SSR ? Date.now() + SSR_DATA_BUDGET_MS : 0 };
    },
    component: Outlet,
    notFoundComponent: () => (
        <section>
            <h1 class="text-2xl font-semibold">Page not found</h1>
            <Link to="/" class="text-muted">
                Return to servers
            </Link>
        </section>
    ),
    errorComponent: AdminRouteError,
    head: () => ({
        meta: [{ title: "Admin | Yin" }, { name: "robots", content: "noindex, nofollow" }],
    }),
});
