<script lang="ts">
	import { onMount } from 'svelte';
	import { invoke } from '@tauri-apps/api/core';
	import Dashboard from '$lib/components/Dashboard.svelte';
	
	let accounts: any[] = [];
	let loading = true;
	let error: string | null = null;

	onMount(async () => {
		try {
			accounts = await invoke('get_accounts');
		} catch (e) {
			error = `Failed to load accounts: ${e}`;
		} finally {
			loading = false;
		}
	});
</script>

<svelte:head>
	<title>RustyAssets - Personal Finance Tracker</title>
</svelte:head>

<main class="app">
	<header>
		<h1>💰 RustyAssets</h1>
		<p>Personal Finance Tracker</p>
	</header>

	{#if loading}
		<div class="loading">Loading your financial data...</div>
	{:else if error}
		<div class="error">
			<h2>Error</h2>
			<p>{error}</p>
			<p><em>Make sure PostgreSQL is running and the database is initialized.</em></p>
		</div>
	{:else}
		<Dashboard {accounts} />
	{/if}
</main>

<style>
	.app {
		min-height: 100vh;
		padding: 2rem;
		background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
		color: white;
	}

	header {
		text-align: center;
		margin-bottom: 2rem;
	}

	header h1 {
		font-size: 3rem;
		margin: 0;
		text-shadow: 2px 2px 4px rgba(0,0,0,0.3);
	}

	header p {
		font-size: 1.2rem;
		opacity: 0.9;
		margin: 0.5rem 0;
	}

	.loading {
		text-align: center;
		font-size: 1.5rem;
		margin-top: 4rem;
	}

	.error {
		background: rgba(255, 255, 255, 0.1);
		padding: 2rem;
		border-radius: 12px;
		text-align: center;
		margin-top: 2rem;
		border: 1px solid rgba(255, 255, 255, 0.2);
	}

	.error h2 {
		color: #ff6b6b;
		margin-top: 0;
	}
</style>
