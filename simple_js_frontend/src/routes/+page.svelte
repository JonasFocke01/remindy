<script lang="ts">
	import { onMount } from 'svelte';
	import { getDayOfYear } from 'date-fns';

	interface Reminder {
		stringifyed: string;
		month: number;
	}

	let reminders: Reminder[] = [];

	onMount(async () => {
		const response = await fetch('http://jonrrrs.duckdns.org:6969/reminders/formatted');
		// const response = await fetch('http://127.0.0.1:6969/reminders/formatted');
		let response_reminders = await response.json();
		reminders = response_reminders.map((reminder: string) => {
			const regex = /(\d{2}).(\d{2}).(\d{4})/; // Regular expression to match the month part
			const match = regex.exec(reminder);
			const month = parseInt(match?.[0].substring(3, 5) ?? '');
			return {
				stringifyed: reminder,
				month: month
			};
		});
	});

	let newName = '';
	let newDescription = '';

	let submit_status: 'Ok' | 'Error' | 'Waiting' | 'Idle' = 'Idle';

	async function handleNewReminder() {
		submit_status = 'Waiting';
		// fetch('http://127.0.0.1:6969/reminders', {
		fetch('http://jonrrrs.duckdns.org:6969/reminders', {
			method: 'POST',
			headers: {
				'Content-Type': 'application/json'
			},
			redirect: 'follow',
			body: JSON.stringify({
				name: newName,
				description: newDescription,
				finish_time: currentDateAsSerializedOffsetDateTime(),
				reminder_type: 'Duration'
			})
		})
			.then((response) => {
				if (response.status === 200) {
					submit_status = 'Ok';
				} else {
					submit_status = 'Error';
				}
			})
			.catch((_error) => {
				submit_status = 'Error';
			});
	}

	function currentDateAsSerializedOffsetDateTime(): number[] {
		let date = new Date();
		let result: number[] = [];

		// year
		result.push(date.getUTCFullYear());
		// day of year
		result.push(getDayOfYear(date) + 1);
		// hours
		result.push(date.getUTCHours() + 1);
		// minutes
		result.push(date.getUTCMinutes());
		// seconds
		result.push(date.getUTCSeconds());
		// milliseconds
		result.push(0);
		// Offset hours
		result.push(1);
		// Offset minutes
		result.push(0);
		// Offset seconds
		result.push(0);

		return result;
	}
</script>

<h1 class="font-bold text-6xl">Remindy</h1>

<h3 class="font-bold my-10">Current reminder list:</h3>
{#each reminders as reminder}
	<div
		class={'border rounded-lg mt-2 p-2 '}
		style:background-color={reminder.month % 2 ? '#1ca51a90' : '#c9460e90'}
	>
		{reminder.stringifyed}
	</div>
{/each}

<form on:submit|preventDefault={handleNewReminder}>
	<h3 class="font-bold mt-10 mb-4">Create new reminder</h3>
	<div class="flex flex-col">
		<label for="newName">Name</label>
		<input
			type="text"
			bind:value={newName}
			class="border w-1/3 text-black"
			on:change={() => (submit_status = 'Idle')}
		/>
		<label for="newDescription">Description (With time and date!)</label>
		<input
			type="text"
			bind:value={newDescription}
			on:change={() => (submit_status = 'Idle')}
			class="border w-1/3 text-black"
		/>
	</div>
	<button class="border w-1/3 mt-4 bg-gray-500">SUBMIT</button>
</form>

{#if submit_status === 'Ok'}
	<p class="bg-green-200 text-black">Successfully created {newName}</p>
{:else if submit_status === 'Error'}
	<p class="bg-red-200 text-black">There was an error when creating {newName}</p>
{:else if submit_status === 'Waiting'}
	<p class="bg-blue-200">Creating {newName}, please wait...</p>
{/if}
