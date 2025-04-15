import type { SourceDiagnostic } from "mathedit-typst-wasm"

function sourceDiagnosticToObject(sourceDiagnostic: SourceDiagnostic) {
	return {
		severity: sourceDiagnostic.severity,
		span: { start: sourceDiagnostic.span.start, end: sourceDiagnostic.span.end },
		message: sourceDiagnostic.message,
		trace: sourceDiagnostic.trace,
		hints: sourceDiagnostic.hints,
	}
}
