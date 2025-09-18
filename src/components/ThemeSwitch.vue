<script setup lang="ts">
import { Icon } from "@iconify/vue";
import { useColorMode } from "@vueuse/core";
import { Button } from "@/components/ui/button";
import {
    DropdownMenu,
    DropdownMenuContent,
    DropdownMenuItem,
    DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";

import { useSettingsStore } from "@/stores/settings";

const settingsStore = useSettingsStore();
const mode = useColorMode();

function handleThemeChange(theme: "light" | "dark" | "auto") {
    mode.value = theme;
    settingsStore.setSetting<string>("theme", theme);
}
</script>

<template>
    <DropdownMenu>
        <DropdownMenuTrigger as-child>
            <Button variant="outline">
                <Icon
                    icon="ph:sun-duotone"
                    class="h-[1.2rem] w-[1.2rem] rotate-0 scale-100 transition-all dark:-rotate-90 dark:scale-0"
                />
                <Icon
                    icon="ph:moon-stars-duotone"
                    class="absolute h-[1.2rem] w-[1.2rem] rotate-90 scale-0 transition-all dark:rotate-0 dark:scale-100"
                />
                <span class="sr-only">{{ "settings.theme.label" }}</span>
            </Button>
        </DropdownMenuTrigger>
        <DropdownMenuContent align="end">
            <DropdownMenuItem @click="handleThemeChange('light')">
                {{ "settings.theme.light" }}
            </DropdownMenuItem>
            <DropdownMenuItem @click="handleThemeChange('dark')">
                {{ "settings.theme.dark" }}
            </DropdownMenuItem>
            <DropdownMenuItem @click="handleThemeChange('auto')">
                {{ "settings.theme.system" }}
            </DropdownMenuItem>
        </DropdownMenuContent>
    </DropdownMenu>
</template>
