<script lang="ts" setup>
import ThemeSwitch from "@/components/ThemeSwitch.vue";
</script>

<template>
    <div class="container mx-auto">
        <div class="space-y-0.5 p-10">
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
            </div>
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
        </div>
    </div>
</template>
