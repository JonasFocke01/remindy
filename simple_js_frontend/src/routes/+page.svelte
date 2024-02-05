<script lang="ts">
	import { onMount } from 'svelte';

	let reminders = [];

	onMount(async () => {
		// const response = await fetch('http://jonrrrs.duckdns.org:6969/reminers');
		const response = await fetch('//188.136.69.200:6969/reminders');
		reminders = await response.json();
	});
</script>

<h1>Welcome to SvelteKit</h1>
<p>Visit <a href="https://kit.svelte.dev">kit.svelte.dev</a> to read the documentation</p>

{#each reminders as reminder}
	<p>
		{reminder.name}
	</p>
{/each}
