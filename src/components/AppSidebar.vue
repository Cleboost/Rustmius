<script setup lang="ts">
import { computed } from "vue";
import { Button } from "@/components/ui/button";
import { Icon } from "@iconify/vue";
import {
    Tooltip,
    TooltipContent,
    TooltipTrigger,
} from "@/components/ui/tooltip";
import { RouterLink, useRoute, useRouter } from "vue-router";
import { useServerInstancesStore } from "@/stores/serverInstances";

const route = useRoute();
const router = useRouter();
const serverInstancesStore = useServerInstancesStore();

function isRouteActive(routePath: string): boolean {
    return routePath === route.path;
}

const routes = [
    { name: "Servers", icon: "ph:house-duotone", path: "/home" },
    { name: "SSH Keys", icon: "ph:key-duotone", path: "/keys" },
];

function closeServerTab(serverId: string, event: Event) {
    event.preventDefault();
    event.stopPropagation();

    console.log("AppSidebar: Closing server tab for:", serverId);
    console.log("Current route:", route.path);

    serverInstancesStore.removeServerInstance(serverId);

    if (route.path === `/server/${serverId}`) {
        router.push("/home");
    }
}

function editServer(serverId: string, event: Event) {
    event.preventDefault();
    event.stopPropagation();

    console.log("AppSidebar: Editing server:", serverId);
    editingServerId.value = serverId;
    editModalOpen.value = true;
}

function handleServerUpdated() {
    console.log("AppSidebar: Server updated, refreshing instances");
    editModalOpen.value = false;
    editingServerId.value = null;
}
</script>

<template>
    <aside
        class="flex flex-col h-dvh pb-2 pt-2 px-2 justify-between bg-sidebar"
    >
        <div class="flex flex-col gap-1">
            <!-- Fixed Routes (Servers, SSH Keys) -->
            <nav class="grid gap-1">
                <Tooltip v-for="route in routes" :key="route.name">
                    <TooltipTrigger as-child>
                        <RouterLink :to="route.path">
                            <Button
                                variant="ghost"
                                size="icon"
                                :class="
                                    isRouteActive(route.path) ? 'bg-muted' : ''
                                "
                                class="rounded-lg"
                                :aria-label="route.name"
                            >
                                <Icon :icon="route.icon" class="size-5" />
                            </Button>
                        </RouterLink>
                    </TooltipTrigger>
                    <TooltipContent side="right" :side-offset="5">
                        {{ route.name }}
                    </TooltipContent>
                </Tooltip>
            </nav>

            <!-- Separator -->
            <div
                v-if="serverInstancesStore.sidebarInstances.length > 0"
                class="border-t border-sidebar-border my-2"
            ></div>

            <nav
                v-if="serverInstancesStore.sidebarInstances.length > 0"
                class="grid gap-1"
            >
                <div
                    v-for="instance in serverInstancesStore.sidebarInstances"
                    :key="instance.id"
                    class="group relative"
                >
                    <Tooltip>
                        <TooltipTrigger as-child>
                            <RouterLink :to="`/server/${instance.id}`">
                                <Button
                                    variant="ghost"
                                    size="icon"
                                    :class="
                                        isRouteActive(`/server/${instance.id}`)
                                            ? 'bg-muted'
                                            : ''
                                    "
                                    class="rounded-lg"
                                    :aria-label="instance.name"
                                >
                                    <Icon icon="lucide:server" class="size-5" />
                                </Button>
                            </RouterLink>
                        </TooltipTrigger>
                        <TooltipContent side="right" :side-offset="5">
                            {{ instance.name }}
                        </TooltipContent>
                    </Tooltip>
                </div>
            </nav>
        </div>

        <!-- Settings at bottom -->
        <div class="">
            <Tooltip>
                <TooltipTrigger as-child>
                    <RouterLink to="/settings">
                        <Button
                            variant="ghost"
                            size="icon"
                            :class="
                                isRouteActive('/settings') ? 'bg-muted' : ''
                            "
                            class="mt-auto rounded-lg"
                            :aria-label="'Settings'"
                        >
                            <Icon icon="lucide:settings" class="size-5" />
                        </Button>
                    </RouterLink>
                </TooltipTrigger>
                <TooltipContent side="right" :side-offset="5">
                    Settings
                </TooltipContent>
            </Tooltip>
        </div>
    </aside>
</template>
