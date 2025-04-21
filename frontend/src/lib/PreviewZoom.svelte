<script lang="ts">
	let { scale = $bindable(1), translation = $bindable([0, 0]), zoomCenter = [0, 0] } = $props()
	let displayedScale = $derived(Math.round(scale * 100))
	const zoomRate = 1.2
	const scrollZoomRate = 1.001

	function zoom(newScale: number) {
		const centerPosBeforeTransform = [
			(zoomCenter[0] - translation[0]) / scale,
			(zoomCenter[1] - translation[1]) / scale,
		]
		scale = newScale
		const centerPosAfterTransform = [
			(zoomCenter[0] - translation[0]) / scale,
			(zoomCenter[1] - translation[1]) / scale,
		]
		translation[0] -= (centerPosBeforeTransform[0] - centerPosAfterTransform[0]) * scale
		translation[1] -= (centerPosBeforeTransform[1] - centerPosAfterTransform[1]) * scale
	}
</script>

<div class="wrapper">
	<button aria-label="-" class='decrement-button' onclick={() => zoom(scale / zoomRate)}>
		<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 10 10">
			<line x1="0" y1="5" x2="10" y2="5" stroke="black" />
		</svg>
	</button>
	<div class="input-wrapper-wrapper"> <!-- i heard you like wrappers, so i wrapped your wrapper (this wrapper keeps the width constant and centers the wrapper underneath, which is there to overlay input + % sign) -->
		<div class="input-wrapper">
			{/* @ts-ignore */ null}
			<input
				style:width='{displayedScale ? String(displayedScale).length : 1}ch'
				oninput={e => displayedScale && zoom(displayedScale / 100)}
				onwheel={e => zoom(scale * Math.pow(scrollZoomRate, -e.deltaY))}
				onkeydown={e => e.key == 'Enter' && e.target!.blur()}
				bind:value={displayedScale} type="number"
			>
			<span class='input-units'>%</span>
		</div>
	</div>
	<button aria-label="+" class='increment-button' onclick={() => zoom(scale * zoomRate)}>
		<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 10 10">
			<line x1="0" y1="5" x2="10" y2="5" stroke="black" />
			<line x1="5" y1="0" x2="5" y2="10" stroke="black" />
		</svg>
	</button>
</div>

<style>
	.wrapper {
		display: flex;
		align-items: center;
		gap: 0.08rem;
	}

	.input-wrapper-wrapper {
		min-width: 41px; /* manually measured so it doesn't resize when having 1-3 digits */
		display: grid;
		place-items: center;
	}

	.input-wrapper {
		display: grid;
		place-items: center;

		> * {
			grid-area: 1 / 1 / 2 / 2
		}
	}

	.input-units {
		pointer-events: none;
		text-align: right;
		width: 100%;
		padding-right: 0.1rem;
	}

	input {
		box-sizing: content-box;
		outline: none;
		text-align: right;
		padding: 0;
		padding-left: 0.3ch;
		padding-right: 1.5ch;
		height: 1.1rem;
	}

	button {
		box-sizing: content-box;
		height: 1.1rem;
		aspect-ratio: 1;
		font-size: 1.2em;
		display: grid;
		place-items: center;
		padding: 0;

		svg {
			height: 60%;
		}
	}
</style>