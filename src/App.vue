<script setup lang="ts">
import AppSidebar from "@/components/AppSidebar.vue";
import { TooltipProvider } from "@/components/ui/tooltip";
</script>

<template>
    <!-- <ConfigProvider> -->
    <!-- <AppTopbar /> -->
    <TooltipProvider>
        <main class="flex h-dvh">
            <AppSidebar />
            <section
                class="bg-background grow p-4 flex flex-col overflow-hidden"
            >
                <router-view />
            </section>
        </main>
    </TooltipProvider>
    <!-- </ConfigProvider> -->
</template>

<style></style>
