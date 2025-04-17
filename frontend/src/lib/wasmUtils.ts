import type { SourceDiagnostic } from 'mathedit-typst-wasm'

export type JsSpan = {
	start: number,
	end: number,
}

export type JsSourceDiagnostic = {
	severity: string,
	span: JsSpan,
	message: string,
	trace: { tracepoint: string, span: JsSpan }[],
	hints: string[]
}

// for console.logging purposes and whatnot
export function sourceDiagnosticToObject(sourceDiagnostic: SourceDiagnostic): JsSourceDiagnostic {
	return {
		severity: sourceDiagnostic.severity,
		span: { start: sourceDiagnostic.span.start, end: sourceDiagnostic.span.end },
		message: sourceDiagnostic.message,
		trace: sourceDiagnostic.trace.map(e => ({
			tracepoint: e.tracepoint,
			span: { start: e.span.start, end: e.span.end }
		})),
		hints: sourceDiagnostic.hints,
	}
}
