use wasm_bindgen::prelude::*;

#[wasm_bindgen(getter_with_clone)]
#[derive(Debug, Clone)]
pub struct Completion {
	pub kind: CompletionKind,
	pub label: String,
	pub apply: Option<String>,
	pub detail: Option<String>,
}

impl Completion {
	pub fn from_typst_completion(completion: typst_ide::Completion) -> Self {
		Self {
			kind: match completion.kind {
				typst_ide::CompletionKind::Syntax => CompletionKind { kind: CompletionType::Syntax, data: None },
				typst_ide::CompletionKind::Func => CompletionKind { kind: CompletionType::Func, data: None },
				typst_ide::CompletionKind::Type => CompletionKind { kind: CompletionType::Type, data: None },
				typst_ide::CompletionKind::Param => CompletionKind { kind: CompletionType::Param, data: None },
				typst_ide::CompletionKind::Constant => CompletionKind { kind: CompletionType::Constant, data: None },
				typst_ide::CompletionKind::Path => CompletionKind { kind: CompletionType::Path, data: None },
				typst_ide::CompletionKind::Package => CompletionKind { kind: CompletionType::Package, data: None },
				typst_ide::CompletionKind::Label => CompletionKind { kind: CompletionType::Label, data: None },
				typst_ide::CompletionKind::Font => CompletionKind { kind: CompletionType::Font, data: None },
				typst_ide::CompletionKind::Symbol(char) => CompletionKind { kind: CompletionType::Symbol, data: Some(char) },
			},
			label: completion.label.to_string(),
			apply: completion.apply.map(|i| i.to_string()),
			detail: completion.detail.map(|i| i.to_string()),
		}
	}
}

#[wasm_bindgen(getter_with_clone)]
#[derive(Debug, Clone)]
pub struct Autocomplete {
	pub position: u32,
	pub completions: Vec<Completion>,
}

impl Autocomplete {
	pub fn from_typst_completion(completion: (usize, Vec<Completion>)) -> Self {
		Self {
			position: completion.0 as u32,
			completions: completion.1,
		}
	}
}

// having both a kind and a type feels confusing, look into that
// (symbol completions have data, but JS doesn't have fancy enums so there are two types here to handle that instead)

#[wasm_bindgen(getter_with_clone)]
#[derive(Debug, Clone)]
pub struct CompletionKind {
	#[wasm_bindgen(js_name = "type")]
	pub kind: CompletionType, // CompletionKind
	pub data: Option<char>,
}

#[wasm_bindgen]
#[derive(Debug, Clone, PartialEq)]
pub enum CompletionType {
	Syntax = "Syntax",
	Func = "Func",
	Type = "Type",
	Param = "Param",
	Constant = "Constant",
	Path = "Path",
	Package = "Package",
	Label = "Label",
	Font = "Font",
	Symbol = "Symbol",
}
