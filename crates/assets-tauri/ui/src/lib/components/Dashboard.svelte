<script lang="ts">
	import { onMount } from 'svelte';
	import { invoke } from '@tauri-apps/api/core';
	import AccountCard from './AccountCard.svelte';
	import BalanceChart from './BalanceChart.svelte';

	export let accounts: any[];

	let balanceSheet: any = null;
	let incomeStatement: any = null;
	let totalAssets = 0;
	let totalLiabilities = 0;
	let netWorth = 0;

	onMount(async () => {
		try {
			// Load financial reports
			balanceSheet = await invoke('get_balance_sheet');
			incomeStatement = await invoke('get_income_statement');

			// Calculate totals
			calculateTotals();
		} catch (e) {
			console.error('Failed to load reports:', e);
		}
	});

	function calculateTotals() {
		const assetAccounts = accounts.filter(a => a.account_type === 'Asset');
		const liabilityAccounts = accounts.filter(a => a.account_type === 'Liability');
		
		// For demo purposes - in real app you'd get balances from backend
		totalAssets = assetAccounts.length * 1000; // Placeholder
		totalLiabilities = liabilityAccounts.length * 500; // Placeholder
		netWorth = totalAssets - totalLiabilities;
	}

	$: accountsByType = accounts.reduce((acc, account) => {
		const type = account.account_type;
		if (!acc[type]) acc[type] = [];
		acc[type].push(account);
		return acc;
	}, {} as Record<string, any[]>);
</script>

<div class="dashboard">
	<!-- Financial Summary Cards -->
	<div class="summary-cards">
		<div class="card assets">
			<h3>Total Assets</h3>
			<div class="amount">€{totalAssets.toLocaleString()}</div>
		</div>
		<div class="card liabilities">
			<h3>Total Liabilities</h3>
			<div class="amount">€{totalLiabilities.toLocaleString()}</div>
		</div>
		<div class="card net-worth">
			<h3>Net Worth</h3>
			<div class="amount">€{netWorth.toLocaleString()}</div>
		</div>
	</div>

	<!-- Charts Section -->
	<div class="charts-section">
		<div class="chart-container">
			<h3>Balance Overview</h3>
			<BalanceChart {accounts} />
		</div>
	</div>

	<!-- Accounts by Type -->
	<div class="accounts-section">
		<h2>Your Accounts</h2>
		{#each Object.entries(accountsByType) as [type, typeAccounts]}
			<div class="account-type-group">
				<h3>{type} Accounts ({typeAccounts.length})</h3>
				<div class="accounts-grid">
					{#each typeAccounts as account}
						<AccountCard {account} />
					{/each}
				</div>
			</div>
		{/each}
	</div>
</div>

<style>
	.dashboard {
		max-width: 1200px;
		margin: 0 auto;
	}

	.summary-cards {
		display: grid;
		grid-template-columns: repeat(auto-fit, minmax(250px, 1fr));
		gap: 1.5rem;
		margin-bottom: 2rem;
	}

	.card {
		background: rgba(255, 255, 255, 0.1);
		backdrop-filter: blur(10px);
		border: 1px solid rgba(255, 255, 255, 0.2);
		border-radius: 12px;
		padding: 1.5rem;
		text-align: center;
		transition: transform 0.2s ease;
	}

	.card:hover {
		transform: translateY(-2px);
	}

	.card h3 {
		margin: 0 0 1rem 0;
		font-size: 1rem;
		opacity: 0.8;
		text-transform: uppercase;
		letter-spacing: 1px;
	}

	.amount {
		font-size: 2rem;
		font-weight: bold;
		margin: 0;
	}

	.assets .amount { color: #4ade80; }
	.liabilities .amount { color: #f87171; }
	.net-worth .amount { color: #60a5fa; }

	.charts-section {
		margin-bottom: 3rem;
	}

	.chart-container {
		background: rgba(255, 255, 255, 0.1);
		backdrop-filter: blur(10px);
		border: 1px solid rgba(255, 255, 255, 0.2);
		border-radius: 12px;
		padding: 2rem;
	}

	.chart-container h3 {
		margin-top: 0;
		text-align: center;
	}

	.accounts-section h2 {
		text-align: center;
		margin-bottom: 2rem;
		font-size: 2rem;
	}

	.account-type-group {
		margin-bottom: 2rem;
	}

	.account-type-group h3 {
		margin-bottom: 1rem;
		padding-bottom: 0.5rem;
		border-bottom: 1px solid rgba(255, 255, 255, 0.2);
	}

	.accounts-grid {
		display: grid;
		grid-template-columns: repeat(auto-fill, minmax(300px, 1fr));
		gap: 1rem;
	}
</style>