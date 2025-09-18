<script setup lang="ts">
import { Button } from "@/components/ui/button";
import { Icon } from "@iconify/vue";
import {
    Tooltip,
    TooltipContent,
    TooltipTrigger,
} from "@/components/ui/tooltip";

const sidebarRoutes = [
    {
        name: "Servers",
        icon: "ph:house-duotone",
        path: "/home",
    },
    {
        name: "SSH Keys",
        icon: "ph:key-duotone",
        path: "/keys",
    },
];
</script>

<template>
    <aside
        class="flex flex-col h-dvh pb-2 pt-2 px-2 justify-between bg-sidebar"
    >
        <nav class="grid gap-1">
            <Tooltip
                v-for="sidebarRoute in sidebarRoutes"
                :key="sidebarRoute.name"
            >
                <TooltipTrigger as-child>
                    <RouterLink :to="sidebarRoute.path">
                        <Button
                            variant="ghost"
                            size="icon"
                            :class="
                                sidebarRoute.path === $route.path
                                    ? 'bg-muted'
                                    : ''
                            "
                            class="rounded-lg"
                            :aria-label="sidebarRoute.name"
                        >
                            <Icon :icon="sidebarRoute.icon" class="size-5" />
                        </Button>
                    </RouterLink>
                </TooltipTrigger>
                <TooltipContent side="right" :side-offset="5">
                    {{ sidebarRoute.name }}
                </TooltipContent>
            </Tooltip>
        </nav>
        <div class="">
            <Tooltip>
                <TooltipTrigger as-child>
                    <RouterLink to="/settings">
                        <Button
                            variant="ghost"
                            size="icon"
                            :class="
                                '/settings' === $route.path ? 'bg-muted' : ''
                            "
                            class="mt-auto rounded-lg"
                            :aria-label="'Label'"
                        >
                            <Icon icon="lucide:server" class="size-5" />
                        </Button>
                    </RouterLink>
                </TooltipTrigger>
                <TooltipContent side="right" :side-offset="5">
                    {{ "settings.label" }}
                </TooltipContent>
            </Tooltip>
        </div>
        <div class="">
            <Tooltip>
                <TooltipTrigger as-child>
                    <RouterLink to="/settings">
                        <Button
                            variant="ghost"
                            size="icon"
                            :class="
                                '/settings' === $route.path ? 'bg-muted' : ''
                            "
                            class="mt-auto rounded-lg"
                            :aria-label="'Label'"
                        >
                            <Icon icon="lucide:settings" class="size-5" />
                        </Button>
                    </RouterLink>
                </TooltipTrigger>
                <TooltipContent side="right" :side-offset="5">
                    {{ "settings.label" }}
                </TooltipContent>
            </Tooltip>
        </div>
    </aside>
</template>
