use std::panic;

use chrono::{DateTime, Datelike, Utc};
use typst::{diag::{self, Warned}, foundations::{self, Datetime}, layout::PagedDocument, syntax::{self, FileId, VirtualPath}, text::{self, FontBook}, utils::LazyHash, Library, World};
use wasm_bindgen::prelude::*;

mod web_structs;
mod web_structs_ide;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

#[wasm_bindgen(getter_with_clone)]
#[derive(Debug, Clone)]
pub struct WebWorld {
    pub source: String,

    // boring stuff to implement world
    library: LazyHash<Library>,
    fonts: Vec<typst::text::Font>,
    book: LazyHash<FontBook>,
    main: FileId,
    now: Option<DateTime<Utc>>,

    compilation_output: Option<Result<PagedDocument, Vec<web_structs::SourceDiagnostic>>>,
    compilation_warnings: Vec<web_structs::SourceDiagnostic>,
}

#[wasm_bindgen]
impl WebWorld {
    /// Create a new web world with no fonts
    pub fn new() -> Self {
        Self {
            source: String::new(),

            library: LazyHash::new(Library::default()),
            fonts: vec![],
            book: LazyHash::new(FontBook::new()),
            main: FileId::new(None, VirtualPath::new("/main.typ")),
            now: None,

            compilation_output: None,
            compilation_warnings: Vec::new(),
        }
    }

    /// Add a font via its binary data
    #[wasm_bindgen(js_name = addFont)]
    pub fn add_font(&mut self, data: &[u8]) {
        // TODO: add woff2 support
        let data = foundations::Bytes::new(data.to_owned());
        let font = text::Font::new(data, 0).unwrap();
        let font_info = font.info().to_owned();
        self.fonts.push(font);
        self.book.push(font_info);
    }

    /// Get errors from the previous compilation
    pub fn errors(&self) -> Vec<web_structs::SourceDiagnostic> {
        let empty_vec = Vec::new();
        self.compilation_output.as_ref().map(|inner| inner.as_ref().err().unwrap_or(&empty_vec)).unwrap_or(&empty_vec).to_owned()
    }

    /// Get warnings from the previous compilation
    pub fn warnings(&self) -> Vec<web_structs::SourceDiagnostic> {
        self.compilation_warnings.clone()
    }
    
    /// Compile the document using the stored source code  
    pub fn compile(&mut self) {
        self.now = Some(Utc::now());
        let Warned { output, warnings } = typst::compile::<PagedDocument>(self);
        self.compilation_output = Some(output.map_err(|v| v.iter().map(|e| web_structs::SourceDiagnostic::from_source_diagnostic_and_world(e, self)).collect()));
        self.compilation_warnings = warnings.iter().map(|w| web_structs::SourceDiagnostic::from_source_diagnostic_and_world(w, self)).collect();
    }
    
    /// Return an SVG of the first page of the compiled document  
    /// Remember to run `compile()` before calling this!
    #[wasm_bindgen(js_name = renderSvg)]
    pub fn render_svg(&self) -> Option<String> {
        if let Some(Ok(output)) = &self.compilation_output {
            Some(typst_svg::svg(&output.pages[0]))
        } else {
            None
        }
    }

    pub fn autocomplete(&self, cursor: u32, explicit: bool) -> Option<web_structs_ide::Autocomplete> {
        // lol
        typst_ide::autocomplete(self, self.compilation_output.as_ref()?.as_ref().ok(), &self.source(FileId::new(None, VirtualPath::new("/main.typ"))).unwrap(), cursor as usize, explicit).and_then(|(position, completions)| Some((position, completions.iter().map(|c| web_structs_ide::Completion::from_typst_completion(c.clone())).collect()))).map(|i| web_structs_ide::Autocomplete::from_typst_completion(i))
    }
}

impl typst::World for WebWorld {
    fn library(&self) ->  &typst::utils::LazyHash<typst::Library> {
        &self.library
    }

    fn book(&self) ->  &LazyHash<typst::text::FontBook> {
        &self.book
    }

    fn main(&self) -> typst::syntax::FileId {
        self.main
    }

    fn source(&self, id:FileId) -> typst::diag::FileResult<typst::syntax::Source> {
        diag::FileResult::Ok(syntax::Source::new(id, self.source.clone()))
    }

    fn file(&self, _id:FileId) -> diag::FileResult<typst::foundations::Bytes> {
        diag::FileResult::Ok(foundations::Bytes::from_string(self.source.clone()))
    }

    fn font(&self, index:usize) -> Option<typst::text::Font> {
        self.fonts.get(index).cloned()
    }

    fn today(&self, offset:Option<i64>) -> Option<foundations::Datetime> {
        // i think i largely copied this from typst's source code
                
        let with_offset = match offset {
            None => self.now?.with_timezone(&chrono::Local).fixed_offset(),
            Some(hours) => {
                let seconds = i32::try_from(hours).ok()?.checked_mul(3600)?;
                self.now?.with_timezone(&chrono::FixedOffset::east_opt(seconds)?)
            }
        };

        Datetime::from_ymd(
            with_offset.year(),
            with_offset.month().try_into().ok()?,
            with_offset.day().try_into().ok()?,
        )
    }
}

impl typst_ide::IdeWorld for WebWorld {
    fn upcast(&self) -> &dyn typst::World {
        self
    }
}

#[wasm_bindgen(start)]
pub fn main() -> Result<(), JsValue> {
    panic::set_hook(Box::new(console_error_panic_hook::hook));
    Ok(())
}
