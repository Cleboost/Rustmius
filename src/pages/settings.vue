<script lang="ts" setup>
import { Icon } from "@iconify/vue";
</script>

<template>
    <div class="container mx-auto">
        <!-- <div class="space-y-0.5 p-10">
            <h2 class="text-2xl font-bold tracking-tight">
                {{ "settings.label" }}
            </h2>
            <p class="text-muted-foreground">
                {{ "settings.description" }}
            </p>
        </div>
        <Separator class="my-6" />
        <div class="space-y-8">
            <div class="flex items-center space-x-2">
                <Label class="text-lg font-medium" for="theme-switch">{{
                    "settings.theme.label"
                }}</Label>
                <ThemeSwitch />
            </div> -->
            <!-- <div class="flex items-center space-x-2">
                <Label class="text-lg font-medium" for="language-select">{{
                    t("languages.label")
                }}</Label>
                <Select id="language-select" v-model="locale">
                    <SelectTrigger>
                        <SelectValue :placeholder="getLanguageLabel(locale)" />
                    </SelectTrigger>
                    <SelectContent>
                        <SelectGroup>
                            <SelectItem
                                v-for="availableLanguage in availableLanguages"
                                :key="availableLanguage.value"
                                :value="availableLanguage.value"
                            >
                                {{ availableLanguage.label }}
                            </SelectItem>
                        </SelectGroup>
                    </SelectContent>
                </Select>
            </div> -->
        <!-- </div> -->
    <div class="fixed inset-0 flex flex-col items-center justify-center">
        <Icon icon="mdi:construction" class="mb-4 text-gray-400" width="128" height="128" />
        <span class="text-gray-400 text-6xl font-medium font-bold">Under Construction</span>
    </div>
    </div>
</template>
