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
				typst_ide::CompletionKind::Syntax => CompletionKind { kind: String::from("Syntax"), data: None },
				typst_ide::CompletionKind::Func => CompletionKind { kind: String::from("Func"), data: None },
				typst_ide::CompletionKind::Type => CompletionKind { kind: String::from("Type"), data: None },
				typst_ide::CompletionKind::Param => CompletionKind { kind: String::from("Param"), data: None },
				typst_ide::CompletionKind::Constant => CompletionKind { kind: String::from("Constant"), data: None },
				typst_ide::CompletionKind::Path => CompletionKind { kind: String::from("Path"), data: None },
				typst_ide::CompletionKind::Package => CompletionKind { kind: String::from("Package"), data: None },
				typst_ide::CompletionKind::Label => CompletionKind { kind: String::from("Label"), data: None },
				typst_ide::CompletionKind::Font => CompletionKind { kind: String::from("Font"), data: None },
				typst_ide::CompletionKind::Symbol(char) => CompletionKind { kind: String::from("Symbol"), data: Some(char) },
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

#[wasm_bindgen(getter_with_clone)]
#[derive(Debug, Clone)]
pub struct CompletionKind {
	pub kind: String, // CompletionKind
	pub data: Option<char>,
}
