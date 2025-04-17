# Mathedit Typst

A website for editing and previewing [Typst](https://github.com/typst/typst) in the browser

Typst was compiled into wasm to render previews, and the [Monaco Editor](https://github.com/microsoft/monaco-editor) is used for the actual editing

To write the wasm code I tried to figure out what `typst/crates/typst-cli` was doing; Typst seems to store stuff and implement things on a "world", so the bulk of the wasm is implementing a "WebWorld" that JS could interface with to use Typst.

If you wanted to do something similar I'd highly recommend [rustwasm.github.io/book](https://rustwasm.github.io/book), which was a great resource when working on this project

Aside from that Vite + Svelte 5 is used to write the website, and know that it's very WIP rn, hopefully I'll work on it more in the future

**in /frontend:**  
`npm install` installs the things  
`npm run dev` runs the website locally  
`npm run build` builds the website; vite will complain about chunk sizes but that's just monaco being monaco  
**in /wasm:**  
`wasm-pack build --release` produces the wasm files  