<script lang="ts">
	import { onMount } from 'svelte';

	let reminders = [];

	onMount(async () => {
		const response = await fetch('http://192.168.2.95:6969/reminders');
		reminders = await response.json();
	});
</script>

<h1 class="bg-green-200">Welcome to SvelteKit</h1>
<p class="mt-96">
	Visit <a href="https://kit.svelte.dev">kit.svelte.dev</a> to read the documentation
</p>
{#each reminders as reminder}
	<p>{reminder.name}</p>
{/each}
