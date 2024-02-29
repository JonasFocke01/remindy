<script lang="ts">
	import { onMount } from 'svelte';

	let reminders = [];

	onMount(async () => {
		// const response = await fetch('http://jonrrrs.duckdns.org:6969/reminders/formatted');
		const response = await fetch('//127.0.0.1:6969/reminders/formatted');
		reminders = await response.json();
	});
</script>

<h1>Welcome to SvelteKit</h1>
<p>Visit <a href="https://kit.svelte.dev">kit.svelte.dev</a> to read the documentation</p>

{#each reminders as reminder}
	<p>
		{reminder.replaceAll(' ޔn ')}
	</p>
{/each}
