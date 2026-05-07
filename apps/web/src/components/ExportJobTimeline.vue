<script setup lang="ts">
import { computed } from "vue";

import { Badge } from "@/components/ui/badge";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { exportJobResultLink, type ExportJob, type ExportJobEvent } from "@/lib/exportApi";
import { allMessages, type Locale } from "@/lib/i18n";

const props = defineProps<{
  locale: Locale;
  job: ExportJob | null;
  events: ExportJobEvent[];
}>();

const labels = computed(() => allMessages()[props.locale]);
const resultLink = computed(() => exportJobResultLink(props.job?.result));
</script>

<template>
  <Card>
    <CardHeader>
      <CardTitle>{{ labels.exportJobTimeline }}</CardTitle>
      <CardDescription v-if="job">
        {{ labels.exportJobStatus }}:
        <Badge variant="secondary">{{ job.status }}</Badge>
      </CardDescription>
      <CardDescription v-else>{{ labels.noExportEvents }}</CardDescription>
    </CardHeader>
    <CardContent class="flex flex-col gap-3">
      <article v-if="job" class="rounded-lg border bg-background/70 p-3 text-sm">
        <p class="font-semibold">{{ job.id }}</p>
        <p class="text-muted-foreground">{{ job.sourcePackId }} → {{ job.targetId }}</p>
      </article>
      <a
        v-if="resultLink"
        class="rounded-lg border bg-background/70 p-3 text-sm font-medium text-primary"
        :href="resultLink"
        rel="noreferrer"
        target="_blank"
      >
        {{ labels.finalExportLink }}: {{ resultLink }}
      </a>
      <p v-if="events.length === 0" class="rounded-lg border bg-background/70 p-3 text-sm text-muted-foreground">
        {{ labels.noExportEvents }}
      </p>
      <article
        v-for="event in events"
        :key="`${event.jobId}:${event.sequence}`"
        class="grid gap-2 rounded-lg border bg-background/70 p-3 text-sm md:grid-cols-[5rem_1fr]"
      >
        <Badge variant="outline" class="w-fit">{{ event.stage }}</Badge>
        <div class="min-w-0">
          <p class="font-medium">{{ event.message }}</p>
          <p class="text-xs text-muted-foreground">
            #{{ event.sequence }} - {{ event.level }} - {{ event.createdAt.split("T")[0] }}
          </p>
        </div>
      </article>
    </CardContent>
  </Card>
</template>
