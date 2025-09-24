<script setup lang="ts">
import { Button } from "@/components/ui/button";
import { Icon } from "@iconify/vue";
import {
    Tooltip,
    TooltipContent,
    TooltipTrigger,
} from "@/components/ui/tooltip";
import { RouterLink, useRoute } from "vue-router";
import { computed } from "vue";
import { useServerInstanceStore } from "@/stores/serverInstance";

const route = useRoute();
const instanceStore = useServerInstanceStore();

const serverInstance = computed(() => {
    return instanceStore.listServerInstances();
});

function isRouteActive(routePath: string): boolean {
    return routePath === route.path;
}

const routes = [
    { name: "Servers", icon: "ph:house-duotone", path: "/home" },
    { name: "SSH Keys", icon: "ph:key-duotone", path: "/keys" },
];
</script>

<template>
    <aside
        class="flex flex-col h-dvh pb-2 pt-2 px-2 justify-between bg-sidebar"
    >
        <div class="flex flex-col gap-1">
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
            <div
                v-if="serverInstance.length > 0"
                class="border-t border-sidebar-border my-2"
            ></div>

            <nav v-if="serverInstance.length > 0" class="grid gap-1">
                <div
                    v-for="instance in serverInstance"
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
                                    :aria-label="
                                        instance.config().getName() || 'N/A'
                                    "
                                >
                                    <Icon icon="lucide:server" class="size-5" />
                                </Button>
                            </RouterLink>
                        </TooltipTrigger>
                        <TooltipContent side="right" :side-offset="5">
                            {{ instance.config().getName() || "N/A" }}
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
