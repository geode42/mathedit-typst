import * as monaco from 'monaco-editor/esm/vs/editor/editor.api'

monaco.editor.defineTheme('mathedit', {
	base: 'vs',
	inherit: true,
	rules: [
		{
			token: 'variable',
			foreground: '#7a18a8',
		},
		{
			token: 'list-marker',
			foreground: '#7a18a8',
		},
		{
			token: 'function',
			foreground: '#3683ff',
		},
		{
			token: 'math.operator',
			foreground: '#096a99',
		},
		{
			token: 'math.delimiter',
			foreground: '#33a230',
		},
		{
			token: 'comment',
			foreground: '#BABABA',
		}
	],
	colors: {
		'editorLineNumber.foreground': '#DDDDDD',
		'editorLineNumber.activeForeground': '#AAAAAA',
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
		{ open: '$', close: '$' }, // not in math would be nice, but the token provider below doesn't do that
		{ open: '"', close: '"', notIn: ['string'] },
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
	// autoCloseBefore: 'a', // how does this work?
})
monaco.languages.setMonarchTokensProvider('typst', {
	unicode: true,
	
	defaultToken: 'invalid',
	
	// source: https://typst.app/docs/reference/foundations/str/#definitions-to-unicode
	stringEscapes: /\\(?:[\\"nrt]|u\{[0-9A-Fa-f]{1,5}\})/,
	mathEscapes: /\\(?:u\{[0-9A-Fa-f]{1,5}\}|.)/, // there are a bunch of math escapes (like $#^_&"\\, and stuff like "\!="), so i'm just matching everything
	symbols: /[+\-*\/^_=!><|]+/,
	mathOperators: ['+', '-', '*', '/', '^', '_', '=', '!=', '>', '<', '>=', '<=', '|'],
	
	tokenizer: {
		root: [
			[/"/,  { token: 'string.quote', bracket: '@open', next: '@string' } ],
			[/\$/,  { token: 'math.delimiter', bracket: '@open', next: '@math' } ],
			{ include: '@whitespace' },
			[/#[a-zA-Z_]?[a-zA-Z0-9_-]*[a-zA-Z](?=\()/, 'function'],
			[/#[a-zA-Z_]?[a-zA-Z0-9_-]*[a-zA-Z]/, 'variable'],
			[/^\s*(-|\+)(.*)$/, ['list-marker', 'text']],
			[/./, "text"],
		],
		string: [
			[/[^\\"]+/,  'string'],
			[/@stringEscapes/, 'string.escape'],
			[/\\./,      'string.escape.invalid'],
			[/"/,        { token: 'string.quote', bracket: '@close', next: '@pop' } ],
		],
		math: [
			// [/[^\\\$"+\-*\/^_=!><|]+/, 'math'], // symbols regex copied here, is there a better way of doing this?
			[/@mathEscapes/, 'math.escape'],
			{ include: '@whitespace' },
			// no invalid escape, see above
			[/@symbols/, { cases: { '@mathOperators': 'math.operator', '@default': '' } }],
			// todo: this isn't accurate, see if \p{L} works with the unicode param (also see https://forum.typst.app/t/what-are-the-rules-for-identifiers-in-typst/665)
			[/[a-zA-Z_][a-zA-Z0-9_-]*[a-zA-Z](?=\()/, 'function'],
			[/[a-zA-Z_][a-zA-Z0-9_-]*[a-zA-Z]/, 'variable'],
			[/[a-zA-Z0-9]/, 'character'],
			[/"/,  { token: 'string.quote', bracket: '@open', next: '@string' } ],
			[/\$/, { token: 'math.delimiter', bracket: '@close', next: '@pop' }],
		],
		comment: [
			[/[^\/*]+/, 'comment' ],
			[/\/\*/,    'comment', '@push' ],    // nested comment
			["\\*/",    'comment', '@pop'  ],
			[/[\/*]/,   'comment' ]
		],
		whitespace: [
			[/[ \t\r\n]+/, 'whitespace'], // should this be all whitespace?
			[/\/\*/,       'comment', '@comment' ],
			[/\/\/.*$/,    'comment'],
		],
	},
})