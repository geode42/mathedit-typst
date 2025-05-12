use wasm_bindgen::prelude::*;

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
    pub tracepoint: Tracepoint,
    pub span: Span,
}

impl TraceEntry {
    fn new_with_range(tracepoint: Tracepoint, range: std::ops::Range<usize>) -> Self {
        Self {
            tracepoint,
            span: Span::from_range(range),
        }
    }
}

#[wasm_bindgen(getter_with_clone)]
#[derive(Debug, Clone)]
pub struct SourceDiagnostic {
    pub severity: SourceDiagnosticSeverity,
    pub span: Span,
    pub message: String,
    pub trace: Vec<TraceEntry>,
    pub hints: Vec<String>,
}

impl SourceDiagnostic {
    pub fn from_source_diagnostic_and_world(
        source_diagnostic: &typst::diag::SourceDiagnostic,
        world: &impl typst::WorldExt,
    ) -> Self {
        Self {
            severity: if source_diagnostic.severity == typst::diag::Severity::Error {
                SourceDiagnosticSeverity::Error
            } else {
                SourceDiagnosticSeverity::Warning
            },
            span: Span::from_range(world.range(source_diagnostic.span).unwrap()),
            message: source_diagnostic.message.to_string(),
            trace: source_diagnostic.trace
                .iter()
                .map(|e| 
                    TraceEntry::new_with_range(
                        Tracepoint::from_typst_tracepoint(&e.v),
                        world.range(e.span).unwrap()
                    ))
                .collect(),
            hints: source_diagnostic.hints.iter().map(String::from).collect(),
        }
    }
}

#[wasm_bindgen]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SourceDiagnosticSeverity {
    Error = "Error",
    Warning = "Warning",
}

/// Part of a diagnostic's trace  
/// `name` will always be present for show rules, might be present for
/// function calls, and will be empty for imports
#[wasm_bindgen]
#[derive(Debug, Clone, PartialEq)]
pub struct Tracepoint {
    variant: TracepointVariant,
    name: Option<String>,
}

#[wasm_bindgen]
impl Tracepoint {
    fn from_typst_tracepoint(tracepoint: &typst::diag::Tracepoint) -> Self {
        match tracepoint {
            typst::diag::Tracepoint::Call(name) => Self {
                variant: TracepointVariant::Call,
                name: name.as_ref().map(String::from)
            },
            typst::diag::Tracepoint::Show(name) => Self {
                variant: TracepointVariant::Show,
                name: Some(name.to_string())
            },
            typst::diag::Tracepoint::Import => Self {
                variant: TracepointVariant::Call,
                name: None
            },
        }
    }

    // messages copied from Typst's source code
    /// Convert the tracepoint into a message
    #[wasm_bindgen(js_name = toMessage)]
    pub fn to_message(&self) -> String {
        match self.variant {
            TracepointVariant::Call => {
                if let Some(name) = &self.name {
                    format!("error occurred in this call of function `{name}`")
                } else {
                    String::from("error occurred in this function call")
                }
            }
            TracepointVariant::Show => {
                if let Some(name) = &self.name {
                    format!("error occurred while applying show rule to this {name}")
                } else {
                    unreachable!()
                }
            }
            TracepointVariant::Import => String::from("error occurred while importing this module"),
            TracepointVariant::__Invalid => unreachable!(),
        }
    }
}

#[wasm_bindgen]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TracepointVariant {
    /// A function call
    Call = "Call",
    /// A show rule application
    Show = "Show",
    /// A module import
    Import = "Import",
}
