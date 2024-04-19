<script lang="ts">
	import { onMount } from 'svelte';
	import { DateInput } from 'date-picker-svelte';
	import { TimePicker } from 'svelte-time-picker';
	import { getDayOfYear } from 'date-fns';

	interface Reminder {
		stringifyed: string;
		month: number;
	}

	let reminders: Reminder[] = [];

	onMount(async () => {
		fetch_reminders();
		setInterval(() => fetch_reminders(), 10000);
	});

	async function fetch_reminders() {
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
	}

	let newName = 'From Handy';
	let newDescription = '';
	let newDate = new Date();

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
				description: `${newDescription}\n                       FIXME!!!\n                        Actual date: ${newDate.toLocaleDateString('de-De')}\n                        Actual time: ${newDate.toLocaleTimeString('de-De')}\n`,
				finish_time: dateAsSerializedOffsetDateTime(newDate),
				reminder_type: 'Time'
			})
		})
			.then((response) => {
				if (response.status === 200) {
					submit_status = 'Ok';
					newDescription = '';
				} else {
					submit_status = 'Error';
				}
			})
			.catch((_error) => {
				submit_status = 'Error';
			});
	}

	function dateAsSerializedOffsetDateTime(date: Date): number[] {
		let result: number[] = [];

		// year
		result.push(date.getUTCFullYear());
		// day of year
		result.push(getDayOfYear(date));
		// hours
		result.push(date.getUTCHours());
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
		class={'border rounded-lg mt-2 p-2'}
		style:background-color={'rgb(0,' +
			reminder.month * 20 +
			',' +
			(255 - reminder.month * 20) +
			')'}
	>
		<p class="w-1/2 truncate">
			{reminder.stringifyed}
		</p>
	</div>
{/each}

<form on:submit|preventDefault={handleNewReminder}>
	<h3 class="font-bold mt-10 mb-4">Create new reminder</h3>
	<div class="flex flex-col">
		<label for="newDescription">Description</label>
		<input
			type="text"
			bind:value={newDescription}
			on:change={() => (submit_status = 'Idle')}
			class="border w-1/3 text-black"
		/>
	</div>
	<div class="flex flex-col">
		<DateInput bind:value={newDate} format="dd.MM.yyyy" />
		<TimePicker
			date={newDate}
			options={{
				bgColor: '#374151',
				is24h: true
			}}
		/>
	</div>
	{#if submit_status === 'Ok'}
		<p class="bg-green-200 text-black">Successfully created {newName}</p>
	{:else if submit_status === 'Error'}
		<p class="bg-red-200 text-black">There was an error when creating {newName}</p>
	{:else if submit_status === 'Waiting'}
		<p class="bg-blue-200">Creating {newName}, please wait...</p>
	{/if}
	<button class="border w-1/3 mt-4 bg-gray-500">SUBMIT</button>
</form>
