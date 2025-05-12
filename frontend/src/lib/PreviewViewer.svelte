<script lang='ts'>
	let { children, scale = $bindable(1), translation = $bindable([0, 0]) } = $props()
	
	const zoomSpeed = 1.001
	const gestureZoomSpeed = 1.01 // pinch-to-zoom on macOS, which emits wheel events with ctrl pressed and a very small deltaY
	const scrollSpeed = 0.25
	const gestureScrollSpeed = 0.9
	const scaleScrollPower = 0.5 // 0.5 feels like a nice middle ground between 0 and 1
	const gestureScaleScrollPower = 0 // consistent panning seems to feel better for gesture scrolling
	const gestureMaxDelta = 100 // max amount deltaX/deltaY before a wheel event stops being considered a gesture, this could go even higher but I'm worried about reaching mouse territory
		
	const autoScrollInitialVelocityTimeFrame = 100 // mouse positions longer than this ms ago will be discarded
	const momentumScrollMinStepDuration = 10 // mouse position entries will be merged until they're at least this ms long (to avoid duration rounding and div0 errors) (browsers only gives timestamps up to 1ms of accuracy)
	const momentumScrollAcceleration = 0.9925 // when auto scrolling velocity is multiplied by this^delta each frame
	const minSpeed = 0.005 // when speed is below this threshold auto scrolling will stop
	const stepWeight = 0.03 // the lower this is, the more "momentum" you need to build up to go fast. If this is 1 the last step's speed will be used
	const momentumScrollMousePositions: number[][] = [] // this should really be called steps

	let containerElement: HTMLDivElement

	function calculateInitialPanVelocity() {
		if (momentumScrollMousePositions.length < 2) return [0, 0] // you need at least two points to get velocity
		const firstStep = momentumScrollMousePositions[0]
		const lastStep = momentumScrollMousePositions.at(-1)!
		let deltaSteps = []
		if (lastStep[0] - firstStep[0] < momentumScrollMinStepDuration) {
			deltaSteps.push({ duration: lastStep[0] - firstStep[0], delta: [lastStep[1] - firstStep[1], lastStep[2] - firstStep[2]] })
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

		// this block calculates the max speed, so that you'd have to build up momentum to go fast
		const velocity = [0, 0]
		for (const step of deltaSteps) {
			// this for loop and the "/ step.duration" interpolate the steps into 1ms chunks so that
			// your mouse's polling rate doesn't affect the physics
			for (let i = 0; i < step.duration; i++) {
				velocity[0] = velocity[0] * (1 - stepWeight) + step.delta[0] / step.duration * stepWeight
				velocity[1] = velocity[1] * (1 - stepWeight) + step.delta[1] / step.duration * stepWeight
			}
		}

		// just final velocity for testing purposes
		// return [deltaSteps.at(-1)!.delta[0] / deltaSteps.at(-1)!.duration, deltaSteps.at(-1)!.delta[1] / deltaSteps.at(-1)!.duration]

		// this is all to say that angle = last-step-angle, speed = min(the-speed-above, last-step-speed)
		const velMag = Math.hypot(...velocity)
		const lastStepSpeed = Math.hypot(...deltaSteps.at(-1)!.delta)
		const lastStepAngle = Math.atan2(deltaSteps.at(-1)!.delta[1], deltaSteps.at(-1)!.delta[0])
		const finalVelMag = Math.min(lastStepSpeed, velMag)
		return [finalVelMag * Math.cos(lastStepAngle), finalVelMag * Math.sin(lastStepAngle)]
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
		const isGesture = Math.abs(e.deltaX) < gestureMaxDelta && Math.abs(e.deltaY) < gestureMaxDelta
		if (e.ctrlKey || e.metaKey) {
			const pointerPosBeforeTransform = [
				((e.clientX - containerElement.offsetLeft) - translation[0]) / scale,
				((e.clientY - containerElement.offsetTop) - translation[1]) / scale,
			]
			scale *= Math.pow(isGesture ? gestureZoomSpeed : zoomSpeed, -e.deltaY)
			const pointerPosAfterTransform = [
				((e.clientX - containerElement.offsetLeft) - translation[0]) / scale,
				((e.clientY - containerElement.offsetTop) - translation[1]) / scale,
			]
			translation[0] -= (pointerPosBeforeTransform[0] - pointerPosAfterTransform[0]) * scale
			translation[1] -= (pointerPosBeforeTransform[1] - pointerPosAfterTransform[1]) * scale
			e.preventDefault()
		} else {
			translation[+e.shiftKey] -= e.deltaX * (isGesture ? gestureScrollSpeed : scrollSpeed) * Math.pow(scale, isGesture ? gestureScaleScrollPower : scaleScrollPower)
			translation[1-+e.shiftKey] -= e.deltaY * (isGesture ? gestureScrollSpeed : scrollSpeed) * Math.pow(scale, isGesture ? gestureScaleScrollPower : scaleScrollPower)
		}
	}}
	onpointerdown={e => {
		if (e.button != 0) return
		dragging = true
		containerElement.setPointerCapture(e.pointerId)
		panPreviousPos = [e.clientX, e.clientY]
	}}
	onpointerup={e => {
		// if (e.button != 0) return doesn't seem to do anything :(
		dragging = false
		containerElement.releasePointerCapture(e.pointerId)
		while (momentumScrollMousePositions.length && (performance.now() - momentumScrollMousePositions[0][0]) > autoScrollInitialVelocityTimeFrame) {
			momentumScrollMousePositions.shift()
		}
		requestAnimationFrame(t => panRecursivelyWithMomentum(calculateInitialPanVelocity(), t, t))
	}}
	onpointermove={e => {
		if (containerElement.hasPointerCapture(e.pointerId)) {
			if (e.getCoalescedEvents) {
				momentumScrollMousePositions.push(...(e.getCoalescedEvents().map(i => [i.timeStamp, i.clientX, i.clientY])))
			} else {
				momentumScrollMousePositions.push([e.timeStamp, e.clientX, e.clientY])
			}
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
		{@render children?.()}
	</div>
</div>

<style>
	.container {
		position: relative;
		height: 100%;

		&.dragging {
			cursor: grabbing;
		}
	}
	.transformed {
		position: absolute;
		transform-origin: top left;
	}
</style>