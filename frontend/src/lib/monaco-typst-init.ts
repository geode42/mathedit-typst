import * as monaco from 'monaco-editor/esm/vs/editor/editor.api'

monaco.editor.defineTheme('mathedit', {
	base: 'vs',
	inherit: true,
	rules: [],
	colors: {
	}
})

monaco.languages.register({ id: 'typst' })
monaco.languages.setLanguageConfiguration('typst', {
	brackets: [['(', ')'], ['{', '}'], ['[', ']']],
	comments: {
		lineComment: '//',
		blockComment: ['/*', '*/']
	},
	autoClosingPairs: [
		{ open: '$', close: '$' },
		{ open: '(', close: ')' },
		{ open: '{', close: '}' },
		{ open: '[', close: ']' },
	],
	onEnterRules: [
		{
			beforeText: /\$/,
			afterText: /\$/,
			action: { indentAction: monaco.languages.IndentAction.IndentOutdent },
		},
	],
})
// monaco.languages.setMonarchTokensProvider('typst', {
//   brackets
// })