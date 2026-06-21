allow_k8s_contexts("kind-yin")

docker_build(
    "yin-rust-dev",
    ".",
    dockerfile="docker/rust.dev.Dockerfile",
    ignore=[
        "apps/auth/node_modules",
        "node_modules",
        "target",
    ],
    live_update=[
        sync("Cargo.toml", "/workspace/Cargo.toml"),
        sync("Cargo.lock", "/workspace/Cargo.lock"),
        sync("crates", "/workspace/crates"),
    ],
)

docker_build(
    "yin-auth-dev",
    ".",
    dockerfile="docker/auth.dev.Dockerfile",
    ignore=[
        "node_modules",
        "target",
    ],
    live_update=[
        sync("apps/auth/src", "/workspace/apps/auth/src"),
        sync("apps/auth/package.json", "/workspace/apps/auth/package.json"),
        sync("apps/auth/tsconfig.json", "/workspace/apps/auth/tsconfig.json"),
        sync("package.json", "/workspace/package.json"),
        sync("pnpm-lock.yaml", "/workspace/pnpm-lock.yaml"),
        sync("pnpm-workspace.yaml", "/workspace/pnpm-workspace.yaml"),
        run(
            "pnpm install --frozen-lockfile",
            trigger=[
                "package.json",
                "pnpm-lock.yaml",
                "pnpm-workspace.yaml",
                "apps/auth/package.json",
            ],
        ),
    ],
)

local_resource(
    "namespace",
    "kubectl apply -f k8s/local/namespace.yaml",
)

k8s_yaml([
    "k8s/local/configmap.yaml",
    "k8s/local/secret.yaml",
    "k8s/local/traefik.yaml",
    "k8s/local/mariadb.yaml",
    "k8s/local/bot.yaml",
    "k8s/local/api.yaml",
    "k8s/local/auth.yaml",
    "k8s/local/ingress.yaml",
])

k8s_resource(
    "traefik",
    resource_deps=["namespace"],
    port_forwards=["8080:8080"],
)

k8s_resource(
    "mariadb",
    resource_deps=["namespace"],
    port_forwards=["3306:3306"],
)

k8s_resource(
    "api",
    resource_deps=["mariadb", "traefik"],
    port_forwards=["3000:3000"],
)

k8s_resource(
    "auth",
    resource_deps=["mariadb", "traefik"],
    port_forwards=["3001:3001"],
)

k8s_resource(
    "bot",
    resource_deps=["mariadb"],
)
