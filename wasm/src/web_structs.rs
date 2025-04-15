use wasm_bindgen::prelude::*;

#[wasm_bindgen(getter_with_clone)]
#[derive(Debug, Clone)]
pub struct CompilationOutput {
    pub warnings: Vec<SourceDiagnostic>,
    pub errors: Vec<SourceDiagnostic>,
}

#[wasm_bindgen]
#[derive(Debug, Clone, Copy)]
pub struct Span {
	// u32 is used so it doesn't show up as a bigint in js
	pub start: u32,
	pub end: u32,
}

impl Span {
	pub fn from_range(range: std::ops::Range<usize>) -> Self {
		Self {
			start: range.start as u32,
			end: range.end as u32,
		}
	}
}

#[wasm_bindgen(getter_with_clone)]
#[derive(Debug, Clone)]
pub struct TraceEntry {
    pub tracepoint: String,
    pub span: Span,
}

impl TraceEntry {
	pub fn new_with_range(tracepoint: String, range: std::ops::Range<usize>) -> Self {
		Self {
			tracepoint,
			span: Span::from_range(range),
		}
	}
}

#[wasm_bindgen(getter_with_clone)]
#[derive(Debug, Clone)]
pub struct SourceDiagnostic {
    pub severity: String,
    pub span: Span,
    pub message: String,
    pub trace: Vec<TraceEntry>,
    pub hints: Vec<String>,
}

impl SourceDiagnostic {
	pub fn from_source_diagnostic_and_world(source_diagnostic: &typst::diag::SourceDiagnostic, world: &impl typst::WorldExt) -> Self {
		Self {
            severity: if source_diagnostic.severity == typst::diag::Severity::Warning { String::from("Warning") } else { String::from("Error") },
            span: Span::from_range(world.range(source_diagnostic.span).unwrap()),
            message: source_diagnostic.message.to_string(),
            // todo: replace e.v.to_string() with something proper
            trace: source_diagnostic.trace.iter().map(|e| TraceEntry::new_with_range(e.v.to_string(), world.range(e.span).unwrap())).collect(),
            hints: source_diagnostic.hints.iter().map(|i| i.to_string()).collect()
        }
	}
}
