<script lang="ts">
	import { onMount } from 'svelte';
	import { Chart, registerables } from 'chart.js';

	export let accounts: any[];

	let chartCanvas: HTMLCanvasElement;
	let chart: Chart;

	onMount(() => {
		Chart.register(...registerables);
		createChart();
		
		return () => {
			if (chart) {
				chart.destroy();
			}
		};
	});

	function createChart() {
		const accountTypes = accounts.reduce((acc, account) => {
			const type = account.account_type;
			acc[type] = (acc[type] || 0) + 1;
			return acc;
		}, {} as Record<string, number>);

		const labels = Object.keys(accountTypes);
		const data = Object.values(accountTypes);
		const colors = [
			'rgba(74, 222, 128, 0.8)',  // Assets - Green
			'rgba(248, 113, 113, 0.8)', // Liabilities - Red  
			'rgba(96, 165, 250, 0.8)',  // Income - Blue
			'rgba(251, 191, 36, 0.8)',  // Expenses - Yellow
			'rgba(167, 139, 250, 0.8)', // Equity - Purple
		];

		chart = new Chart(chartCanvas, {
			type: 'doughnut',
			data: {
				labels: labels,
				datasets: [{
					data: data,
					backgroundColor: colors.slice(0, labels.length),
					borderColor: colors.slice(0, labels.length).map(color => 
						color.replace('0.8', '1')
					),
					borderWidth: 2
				}]
			},
			options: {
				responsive: true,
				maintainAspectRatio: false,
				plugins: {
					legend: {
						position: 'bottom',
						labels: {
							color: 'white',
							padding: 20,
							font: {
								size: 14
							}
						}
					},
					tooltip: {
						callbacks: {
							label: function(context) {
								const label = context.label || '';
								const value = context.parsed;
								const total = data.reduce((a, b) => a + b, 0);
								const percentage = ((value / total) * 100).toFixed(1);
								return `${label}: ${value} accounts (${percentage}%)`;
							}
						}
					}
				}
			}
		});
	}

	// Recreate chart when accounts change
	$: if (chart && accounts) {
		chart.destroy();
		createChart();
	}
</script>

<div class="chart-wrapper">
	<canvas bind:this={chartCanvas}></canvas>
</div>

<style>
	.chart-wrapper {
		position: relative;
		height: 300px;
		width: 100%;
	}
</style>