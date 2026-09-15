<script lang="ts">
  import ProjectCard from './ProjectCard.svelte';
  import type { Project } from '../types';

  interface Props {
    projects: Project[];
  }

  let { projects }: Props = $props();
</script>

<div class="grid gap-3 p-6" style="grid-template-columns: repeat(auto-fill, minmax(280px, 1fr));">
  {#each projects as project (project.id)}
    <ProjectCard {project} />
  {/each}
</div>
