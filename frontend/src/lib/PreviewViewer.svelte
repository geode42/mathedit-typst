<script lang='ts'>
	let { svg = $bindable(''), scale = $bindable(1), translation = $bindable([0, 0]) } = $props()
	const zoomSpeed = 1.001
	const scrollSpeed = 0.25
	const scaleScrollPower = 0.5 // 0.5 feels like a nice middle ground between 0 and 1
	
	const autoScrollInitialVelocityTimeFrame = 100 // mouse positions longer than this ms ago will be discarded
	const momentumScrollMinStepDuration = 3 // mouse position records will be merged until they're at least this ms long (to avoid rounding and div0 errors)
	const momentumScrollAcceleration = 0.993 // when auto scrolling velocity is multiplied by this^delta each frame
	const minSpeed = 0.01 // when speed is below this threshold auto scrolling will stop
	const stepWeight = 0.25 // when calculating initial auto scroll velocity, each step is multiplied by this^duration and added to the existing velocity is multiplied by 1-that
	const momentumScrollMousePositions: number[][] = [] // this should really be called steps

	let containerElement: HTMLDivElement

	function calculateInitialPanVelocity() {
		if (!momentumScrollMousePositions.length) return [0, 0]
		const firstStep = momentumScrollMousePositions[0]
		const lastStep = momentumScrollMousePositions.at(-1)!
		let deltaSteps = []
		if (lastStep[0] - firstStep[0] < momentumScrollMinStepDuration) {
			deltaSteps.push({ duration: lastStep[0] - firstStep[0], delta: [lastStep[0] - firstStep[0], lastStep[1] - firstStep[1]] })
		} else {
			const mergedMouseSteps = [{ startTime: firstStep[0], endTime: firstStep[0], startPos: [firstStep[1], firstStep[2]], endPos: [0, 0] }]
			for (const step of momentumScrollMousePositions) {
				if (mergedMouseSteps.at(-1)!.endTime - mergedMouseSteps.at(-1)!.startTime < momentumScrollMinStepDuration) {
					mergedMouseSteps.at(-1)!.endTime = step[0]
					mergedMouseSteps.at(-1)!.endPos = [step[1], step[2]]
				} else {
					mergedMouseSteps.push({ startTime: step[0], endTime: step[0], startPos: [step[1], step[2]], endPos: [step[1], step[2]] })
				}
			}
	
			deltaSteps = mergedMouseSteps.map(s => ({ duration: s.endTime - s.startTime, delta: [s.endPos[0] - s.startPos[0], s.endPos[1] - s.startPos[1]] }))
			// if the last step is too long, merges it with the second-to-last
			// the last step should needs to be the most accurate anyway so the extra time's probably fine
			if (deltaSteps.at(-1)!.duration < momentumScrollMinStepDuration) {
				deltaSteps.at(-2)!.duration += deltaSteps.at(-1)!.duration
				deltaSteps.at(-2)!.delta[0] += deltaSteps.at(-1)!.delta[0]
				deltaSteps.at(-2)!.delta[1] += deltaSteps.at(-1)!.delta[1]
				deltaSteps.pop()
			}
		}

		const velocity = [0, 0]
		for (const step of deltaSteps) {
			const weight = Math.pow(stepWeight, step.duration)
			velocity[0] = velocity[0] * (1 - weight) + step.delta[0] * weight
			velocity[1] = velocity[1] * (1 - weight) + step.delta[1] * weight
		}

		return velocity
	}

	function panRecursivelyWithMomentum(speed: number[], previousTimestamp: number, previousPreviousTimestamp: number) {
		const delta = (document.timeline.currentTime as number - previousPreviousTimestamp)
		translation[0] += speed[0] * delta
		translation[1] += speed[1] * delta
		if (Math.hypot(...speed) >= minSpeed) requestAnimationFrame(() => panRecursivelyWithMomentum(speed.map(i => i * Math.pow(momentumScrollAcceleration, delta)), document.timeline.currentTime as number, previousTimestamp))
	}

	let panPreviousPos = [0, 0]
	let dragging = $state(false)
</script>

<div
	bind:this={containerElement}
	class='container'
	class:dragging={dragging}
	onwheel={e => {
		if (e.ctrlKey) {
			const pointerPosBeforeTransform = [
				((e.clientX - containerElement.offsetLeft) - translation[0]) / scale,
				((e.clientY - containerElement.offsetTop) - translation[1]) / scale,
			]
			scale *= Math.pow(zoomSpeed, -e.deltaY)
			const pointerPosAfterTransform = [
				((e.clientX - containerElement.offsetLeft) - translation[0]) / scale,
				((e.clientY - containerElement.offsetTop) - translation[1]) / scale,
			]
			translation[0] -= (pointerPosBeforeTransform[0] - pointerPosAfterTransform[0]) * scale
			translation[1] -= (pointerPosBeforeTransform[1] - pointerPosAfterTransform[1]) * scale
			e.preventDefault()
		} else {
			translation[+e.shiftKey] -= e.deltaX * scrollSpeed * Math.pow(scale, scaleScrollPower)
			translation[1-+e.shiftKey] -= e.deltaY * scrollSpeed * Math.pow(scale, scaleScrollPower)
		}
	}}
	onpointerdown={e => {
		dragging = true
		containerElement.setPointerCapture(e.pointerId)
		panPreviousPos = [e.clientX, e.clientY]
	}}
	onpointerup={e => {
		dragging = false
		containerElement.releasePointerCapture(e.pointerId)
		while (momentumScrollMousePositions.length && (performance.now() - momentumScrollMousePositions[0][0]) > autoScrollInitialVelocityTimeFrame) {
			momentumScrollMousePositions.shift()
		}
		requestAnimationFrame(t => panRecursivelyWithMomentum(calculateInitialPanVelocity(), t, t))
	}}
	onpointermove={e => {
		if (containerElement.hasPointerCapture(e.pointerId)) {
			momentumScrollMousePositions.push(...(e.getCoalescedEvents().map(i => [i.timeStamp, i.clientX, i.clientY])))
			while (momentumScrollMousePositions.length && (performance.now() - momentumScrollMousePositions[0][0]) > autoScrollInitialVelocityTimeFrame) {
				momentumScrollMousePositions.shift()
			}
			translation[0] += e.clientX - panPreviousPos[0]
			translation[1] += e.clientY - panPreviousPos[1]
			panPreviousPos = [e.clientX, e.clientY]
		}
	}}
>
	<div class="transformed" style:transform={`translate(${translation[0]}px, ${translation[1]}px) scale(${scale})`}>
		{@html svg}
	</div>
</div>

<style>
	.container {
		position: relative;
		flex: 1 1 0;
		overflow: hidden;

		&.dragging {
			cursor: grabbing;
		}
	}
	.transformed {
		position: absolute;
		transform-origin: top left;

		:global(svg) {
			/* should these be px? */
			box-shadow: 0 0 0.7rem #00000015;
			/* border-radius: 1rem; */
		}
	}
</style>