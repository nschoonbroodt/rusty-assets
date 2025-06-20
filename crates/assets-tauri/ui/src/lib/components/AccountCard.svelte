<script lang="ts">
	export let account: any;

	function formatAccountType(type: string) {
		return type.charAt(0).toUpperCase() + type.slice(1);
	}

	function getAccountIcon(type: string) {
		switch (type) {
			case 'Asset': return '💰';
			case 'Liability': return '💳';
			case 'Income': return '💵';
			case 'Expense': return '🛒';
			case 'Equity': return '📈';
			default: return '📊';
		}
	}
</script>

<div class="account-card">
	<div class="account-header">
		<div class="account-icon">
			{getAccountIcon(account.account_type)}
		</div>
		<div class="account-info">
			<h4>{account.name}</h4>
			<p class="account-path">{account.full_path || account.name}</p>
		</div>
	</div>
	
	<div class="account-details">
		<div class="detail-row">
			<span class="label">Type:</span>
			<span class="value">{formatAccountType(account.account_type)}</span>
		</div>
		
		{#if account.account_subtype}
			<div class="detail-row">
				<span class="label">Subtype:</span>
				<span class="value">{account.account_subtype}</span>
			</div>
		{/if}
		
		{#if account.symbol}
			<div class="detail-row">
				<span class="label">Symbol:</span>
				<span class="value">{account.symbol}</span>
			</div>
		{/if}
		
		{#if account.quantity}
			<div class="detail-row">
				<span class="label">Quantity:</span>
				<span class="value">{account.quantity}</span>
			</div>
		{/if}
		
		<div class="detail-row">
			<span class="label">Currency:</span>
			<span class="value">{account.currency}</span>
		</div>
		
		<div class="detail-row">
			<span class="label">Status:</span>
			<span class="value {account.is_active ? 'active' : 'inactive'}">
				{account.is_active ? 'Active' : 'Inactive'}
			</span>
		</div>
	</div>
</div>

<style>
	.account-card {
		background: rgba(255, 255, 255, 0.1);
		backdrop-filter: blur(10px);
		border: 1px solid rgba(255, 255, 255, 0.2);
		border-radius: 12px;
		padding: 1.5rem;
		transition: transform 0.2s ease, box-shadow 0.2s ease;
	}

	.account-card:hover {
		transform: translateY(-2px);
		box-shadow: 0 8px 25px rgba(0, 0, 0, 0.2);
	}

	.account-header {
		display: flex;
		align-items: center;
		margin-bottom: 1rem;
		padding-bottom: 1rem;
		border-bottom: 1px solid rgba(255, 255, 255, 0.1);
	}

	.account-icon {
		font-size: 2rem;
		margin-right: 1rem;
	}

	.account-info h4 {
		margin: 0;
		font-size: 1.2rem;
		font-weight: 600;
	}

	.account-path {
		margin: 0.25rem 0 0 0;
		font-size: 0.9rem;
		opacity: 0.7;
	}

	.account-details {
		space-y: 0.5rem;
	}

	.detail-row {
		display: flex;
		justify-content: space-between;
		align-items: center;
		margin-bottom: 0.5rem;
	}

	.label {
		font-size: 0.9rem;
		opacity: 0.8;
	}

	.value {
		font-weight: 500;
	}

	.value.active {
		color: #4ade80;
	}

	.value.inactive {
		color: #f87171;
	}
</style>