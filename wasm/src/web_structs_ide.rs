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
				typst_ide::CompletionKind::Syntax => CompletionKind { variant: CompletionVariant::Syntax, data: None },
				typst_ide::CompletionKind::Func => CompletionKind { variant: CompletionVariant::Func, data: None },
				typst_ide::CompletionKind::Type => CompletionKind { variant: CompletionVariant::Type, data: None },
				typst_ide::CompletionKind::Param => CompletionKind { variant: CompletionVariant::Param, data: None },
				typst_ide::CompletionKind::Constant => CompletionKind { variant: CompletionVariant::Constant, data: None },
				typst_ide::CompletionKind::Path => CompletionKind { variant: CompletionVariant::Path, data: None },
				typst_ide::CompletionKind::Package => CompletionKind { variant: CompletionVariant::Package, data: None },
				typst_ide::CompletionKind::Label => CompletionKind { variant: CompletionVariant::Label, data: None },
				typst_ide::CompletionKind::Font => CompletionKind { variant: CompletionVariant::Font, data: None },
				typst_ide::CompletionKind::Symbol(char) => CompletionKind { variant: CompletionVariant::Symbol, data: Some(char) },
			},
			label: completion.label.to_string(),
			apply: completion.apply.map(String::from),
			detail: completion.detail.map(String::from),
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

/// `data` is a character for the `Symbol` variant and empty otherwise
#[wasm_bindgen(getter_with_clone)]
#[derive(Debug, Clone)]
pub struct CompletionKind {
	pub variant: CompletionVariant,
	pub data: Option<char>,
}

#[wasm_bindgen]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CompletionVariant {
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
