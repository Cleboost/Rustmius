<script setup lang="ts">
import { Icon } from "@iconify/vue";

defineProps<{
    name: string;
    desc: string;
    icon: string;
    disabled?: boolean;
    tag?: string;
}>();

defineEmits<{
    (e: "click"): void;
}>();
</script>

<template>
    <div
        :class="[
            'border rounded-lg p-4 transition-colors',
            disabled 
                ? 'opacity-50 cursor-not-allowed bg-muted/20' 
                : 'hover:bg-muted/50 cursor-pointer'
        ]"
        @click="!disabled && $emit('click')"
    >
        <div class="flex items-center gap-3">
            <Icon 
                :icon="icon" 
                :class="[
                    'w-8 h-8',
                    disabled ? 'text-muted-foreground' : 'text-primary'
                ]"
            />
            <div>
                <div class="flex items-center gap-2">
                    <h3 :class="[
                        'font-semibold',
                        disabled ? 'text-muted-foreground' : ''
                    ]">{{ name }}</h3>
                    <span 
                        v-if="tag" 
                        class="px-2.5 py-1 text-xs font-semibold bg-gradient-to-r from-orange-400 to-orange-500 text-white rounded-full shadow-sm border border-orange-300/20"
                    >
                        {{ tag }}
                    </span>
                </div>
                <p class="text-sm text-muted-foreground">{{ desc }}</p>
            </div>
        </div>
    </div>
</template>