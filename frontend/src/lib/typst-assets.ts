export async function importBundledFonts(): Promise<Uint8Array[]> {
	return (await Promise.allSettled([
		// load all the fonts in the typst-fonts directory, these are all bundled with typst
		"typst-fonts/DejaVuSansMono.ttf",
		"typst-fonts/DejaVuSansMono-Oblique.ttf",
		"typst-fonts/DejaVuSansMono-Bold.ttf",
		"typst-fonts/DejaVuSansMono-BoldOblique.ttf",

		"typst-fonts/LibertinusSerif-Regular.otf",
		"typst-fonts/LibertinusSerif-Italic.otf",
		"typst-fonts/LibertinusSerif-Semibold.otf",
		"typst-fonts/LibertinusSerif-SemiboldItalic.otf",
		"typst-fonts/LibertinusSerif-Bold.otf",
		"typst-fonts/LibertinusSerif-BoldItalic.otf",

		"typst-fonts/NewCM10-Regular.otf",
		"typst-fonts/NewCM10-Italic.otf",
		"typst-fonts/NewCM10-Bold.otf",
		"typst-fonts/NewCM10-BoldItalic.otf",
		"typst-fonts/NewCMMath-Regular.otf",
		"typst-fonts/NewCMMath-Book.otf",
		"typst-fonts/NewCMMath-Bold.otf",
	].map(url => new Promise(async r => {
		const data = new Uint8Array(await (await fetch(url)).arrayBuffer())
		r(data)
	})))).map(i => i.status == 'fulfilled' ? i.value : (console.error(i.reason), null)) as Uint8Array[]
}
