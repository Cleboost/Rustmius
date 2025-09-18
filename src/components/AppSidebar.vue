<script setup lang="ts">
import { Button } from "@/components/ui/button";
import { Icon } from "@iconify/vue";
import {
    Tooltip,
    TooltipContent,
    TooltipTrigger,
} from "@/components/ui/tooltip";
import { sidebarRoutes } from "@/stores/tabs";
import { useConsolesStore } from "@/stores/consoles";
import { RouterLink } from "vue-router";
const consoles = useConsolesStore();
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
        <div class="flex flex-col gap-1">
            <RouterLink
                v-for="s in consoles.list()"
                :key="s.serverId"
                :to="`/server/${s.serverId}/console`"
            >
                <Button
                    variant="ghost"
                    size="icon"
                    :class="`rounded-lg ${$route.params.id === s.serverId ? 'bg-muted' : ''}`"
                    :aria-label="s.serverId"
                    :title="s.serverId"
                >
                    <Icon icon="lucide:server" class="size-5" />
                </Button>
            </RouterLink>
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
