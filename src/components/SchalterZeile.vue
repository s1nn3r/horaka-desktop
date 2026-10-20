<script setup lang="ts">
// Eine Zeile in einer Einstellungskarte: Titel, kurzer Text, Schalter rechts.
defineProps<{ titel: string; text?: string; an: boolean }>();
const emit = defineEmits<{ aendern: [an: boolean] }>();
</script>

<template>
    <label class="relative flex cursor-default items-center gap-3 px-3 py-2.5">
        <span class="min-w-0 flex-1">
            <span class="block text-[13px] font-medium">{{ titel }}</span>
            <span v-if="text" class="block text-[12px] leading-snug text-muted-foreground">{{ text }}</span>
        </span>
        <input type="checkbox" role="switch" class="peer sr-only" :checked="an" @change="emit('aendern', ($event.target as HTMLInputElement).checked)" />
        <span
            aria-hidden="true"
            class="relative h-5 w-9 shrink-0 rounded-full bg-muted-foreground/35 transition-colors peer-checked:bg-primary peer-focus-visible:outline-2 peer-focus-visible:outline-offset-2 peer-focus-visible:outline-ring motion-reduce:transition-none after:absolute after:top-0.5 after:left-0.5 after:size-4 after:rounded-full after:bg-white after:shadow-sm after:transition-transform peer-checked:after:translate-x-4 motion-reduce:after:transition-none"
        />
    </label>
</template>
