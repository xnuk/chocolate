import { State as Chocolate } from '../../build/wasm/chocolate.js'

import {
	source as sourceEl,
	sourceView,
	stack as stackEl,
	error as errorEl,
	input as inputEl,
	buttonNext,
	buttonRun,
	buttonStop,
	runMetadata,
	main,
	output as outputEl,
} from './dom.ts'

const DP = ['오른쪽', '아래쪽', '왼쪽', '위쪽']
const CC = ['왼쪽', '오른쪽']

const displayCmd = (cmd: string, len: number) => {
	if (cmd === '\0') return '없음'
	if (cmd === 'P') return `P(${len})`
	return cmd
}

class State {
	codeMap: HTMLSpanElement[][]
	beforeIndex = 0
	beforeStack = new BigInt64Array(0)

	constructor(
		code: string,
		private chocolate: Chocolate,
	) {
		const areaIndices: Uint32Array[] = []

		const flattenArray = chocolate.area_indices_flatten()
		const flattenLen = flattenArray.length
		let idx = 0
		while (idx < flattenLen) {
			const len = flattenArray[idx] ?? 0
			if (len === 0) break
			idx += 1
			areaIndices.push(flattenArray.subarray(idx, idx + len))
			idx += len
		}

		const codeMap: HTMLSpanElement[][] = [[]]

		// oxlint-disable-next-line oxc/no-map-spread: this called only once
		const children = code.split('\n').flatMap((line, row) => {
			const ret: HTMLElement[] = [...line].map((text, col) => {
				const span = document.createElement('span')
				span.textContent = text

				const index = areaIndices[row]?.[col] ?? 0
				if (index > 0) (codeMap[index] ||= []).push(span)

				return span
			})
			ret.push(document.createElement('br'))
			return ret
		})

		sourceView.append(...children)

		this.codeMap = codeMap
	}

	get finished() {
		return this.chocolate.finished
	}

	/// returns step is successed
	// oxlint-disable-next-line max-lines-per-function
	step(): boolean {
		if (this.chocolate.finished) {
			runMetadata.textContent = '종료됨'
			return false
		}

		// oxlint-disable-next-line curly: oxlint sucks
		for (const before of this.codeMap[this.beforeIndex] ?? []) {
			before.className = ''
		}

		const currentIndex = this.chocolate.next_index()
		// oxlint-disable-next-line curly: oxlint sucks
		for (const current of this.codeMap[currentIndex] ?? []) {
			current.className = 'active'
		}

		this.beforeIndex = currentIndex

		const currentCmd = String.fromCodePoint(this.chocolate.next_cmd())
		const currentLen = this.chocolate.next_len()

		let metadata = `현재 명령: ${displayCmd(currentCmd, currentLen).padEnd(5)}`

		const error = this.chocolate.step()
		if (error.length === 0) {
			const dp = DP[this.chocolate.dp]
			const cc = CC[this.chocolate.cc]

			const nextCmd = String.fromCodePoint(this.chocolate.next_cmd())
			const nextLen = this.chocolate.next_len()

			if (currentCmd !== nextCmd) {
				const next = displayCmd(nextCmd, nextLen)
				metadata += ` | DP: ${dp} | CC: ${cc} | 다음 명령: ${next}`
			} else {
				metadata += ' | 다음 명령: 종료됨'
			}

			const out = new TextDecoder().decode(this.chocolate.output())
			if (outputEl.textContent !== out) outputEl.textContent = out

			const stack = this.chocolate.stack()
			const list = [...stackEl.getElementsByTagName('li')]
			const next: HTMLLIElement[] = []
			for (let idx = 0; idx < stack.length; idx += 1) {
				const val = stack[idx] ?? 0n
				const li = list[idx]
				const before = this.beforeStack[idx]
				if (li == null) {
					const newLi = document.createElement('li')
					newLi.textContent = String(val)
					next.push(newLi)
					continue
				}
				if (before !== val) li.textContent = String(val)
			}
			for (const el of list.slice(stack.length)) el.remove()
			stackEl.append(...next)
			this.beforeStack = stack as BigInt64Array<ArrayBuffer>
		} else {
			errorEl.textContent = new TextDecoder().decode(error)
		}
		runMetadata.textContent = metadata

		return error.length === 0
	}

	[Symbol.dispose]() {
		this.chocolate[Symbol.dispose]()
	}
}

let globalState: State | null = null
let stopRunning: (() => void) | null = null

const makeState = (): State | null => {
	const textEncoder = new TextEncoder()
	const code = textEncoder.encode(sourceEl.value)
	const input = textEncoder.encode(inputEl.value)
	const errorOutput = new Uint8Array(256)

	const chocolate = Chocolate.from_source_and_input(code, input, errorOutput)
	if (chocolate == null) {
		let len = 0
		for (; len < 256; len += 1) if (errorOutput[len] === 0) break

		const error = new TextDecoder().decode(errorOutput.subarray(0, len))
		errorEl.textContent = error
	}
	return chocolate == null ? null : new State(sourceEl.value, chocolate)
}

const toEditing = () => {
	if (globalState != null) {
		globalState[Symbol.dispose]()
		globalState = null
	}
	if (stopRunning != null) {
		stopRunning()
		stopRunning = null
	}

	sourceView.innerHTML = ''
	stackEl.innerHTML = ''
	errorEl.textContent = ''
	outputEl.textContent = ''
	main.className = 'editing'

	inputEl.disabled = false

	buttonNext.disabled = false
	buttonRun.disabled = false
	buttonRun.textContent = '실행'
	buttonStop.disabled = true
}

const toRunning = () => {
	globalState = makeState()
	if (globalState == null) return

	errorEl.textContent = ''
	main.className = 'running'

	const { width, height } = sourceEl.style
	sourceView.style.width = width === '' ? '0px' : width
	sourceView.style.height = height === '' ? '0px' : height

	inputEl.disabled = true

	buttonNext.disabled = false
	buttonRun.disabled = false
	buttonStop.disabled = false
}

const step = (): boolean => {
	if (globalState == null) return false
	return globalState.step()
}

buttonNext.addEventListener('click', () => {
	if (globalState == null) toRunning()
	if (!step()) {
		buttonNext.disabled = true
		buttonRun.disabled = true
	}
})

buttonRun.addEventListener('click', () => {
	if (stopRunning == null) {
		if (globalState == null) toRunning()

		let interval = 0
		const stop = () => {
			clearInterval(interval)
			stopRunning = null

			if (globalState != null && globalState.finished) {
				buttonNext.disabled = true
				buttonRun.disabled = true
				buttonRun.textContent = '실행'
			}
		}
		interval = setInterval(() => {
			if (!step()) stop()
		}) as unknown as number

		stopRunning = stop

		buttonRun.textContent = '일시정지'
		buttonNext.disabled = true
	} else {
		stopRunning()
		stopRunning = null
		buttonRun.textContent = '실행'
		buttonNext.disabled = false
	}
})

buttonStop.addEventListener('click', () => {
	toEditing()
})
