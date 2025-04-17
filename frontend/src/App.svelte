<script lang="ts">
  import * as wasm from 'mathedit-typst-wasm' // todo: see if top level await on wasm (see npm package) could be removed?
  import * as monaco from 'monaco-editor/esm/vs/editor/editor.api'
  import { onMount } from 'svelte'
  import './lib/userWorker'
  import './lib/monaco-typst-init'
  import { importBundledFonts } from './lib/typst-assets'
  import PreviewViewer from './lib/PreviewViewer.svelte'
  import PreviewZoom from './lib/PreviewZoom.svelte'
  import { type JsSourceDiagnostic } from './lib/wasmUtils'
  import { registerAutocomplete } from './lib/monaco-typst-init'

  const defaultSource = [
    'Some math in Typst',
    '',
    '$(m v^2) / r = q v times B$',
    ''
  ].join('\n')

  const beforeUnloadHandler = (e: BeforeUnloadEvent) => e.preventDefault()

  let source = $state(defaultSource)
  let previewSvg = $state('')
  let previewSvgElement = $derived(new DOMParser().parseFromString(previewSvg ?? '<div></div>', 'text/html').body.childNodes[0])
  let previewScale = $state(1)
  let previewTranslation = $state([0, 0])
  let previewContainer: HTMLDivElement
  let svgWrapper: HTMLDivElement
  
  let renderedOnce = -1 // -1 initially, 0 for the first fake update the effect below gets, 1 when it rendered the typst once
  let previewCenter = $state([0, 0])

  function updatePreviewCenter() {
    previewCenter = [previewContainer.offsetWidth / 2, previewContainer.offsetHeight / 2]
  }

  $effect(() => {
    source != defaultSource
      ? addEventListener('beforeunload', beforeUnloadHandler)
      : removeEventListener('beforeunload', beforeUnloadHandler)
  })

  $effect(() => {
    svgWrapper.replaceChildren(previewSvgElement)
    setTimeout(() => {
      if (renderedOnce != 1) {
        renderedOnce++
        if (renderedOnce == 1) {
          zoomFit()
          translateTop()
          updatePreviewCenter()
        }
      }
    })
  })

  let webWorldLoaded = false
  const webWorld = wasm.WebWorld.new()
  ;(async () => {
    (await importBundledFonts()).forEach(f => webWorld.addFont(f))
    webWorldLoaded = true
    renderPreview()
  })()


  function renderPreview() {
    if (!webWorldLoaded) return
    webWorld.source = source
    webWorld.compile()
    const svg = webWorld.renderSvg()

    if (svg) previewSvg = svg

    const warnings: JsSourceDiagnostic[] = webWorld.warnings()
    const errors: JsSourceDiagnostic[] = webWorld.errors()

    // in IMarkerData, `code` and `source` get added in light gray right after the message
    // src: https://code.visualstudio.com/api/references/vscode-api#Diagnostic

    const getMarkerSpan = (diagnostic: JsSourceDiagnostic) => {
      const start = editor.getModel()!.getPositionAt(diagnostic.span.start)
      const end = editor.getModel()!.getPositionAt(diagnostic.span.end)
      return {
        startLineNumber: start.lineNumber,
        startColumn: start.column,
        endLineNumber: end.lineNumber,
        endColumn: end.column,
      }
    }

    const warningMarkers = warnings.flatMap(i => {
      return [
        {
          severity: monaco.MarkerSeverity.Warning,
          message: i.message,
          ...getMarkerSpan(i),
        },
        ...i.hints.map(h => ({
          severity: monaco.MarkerSeverity.Hint,
          message: h,
          ...getMarkerSpan(i),
        }))
      ]
    })
    const errorMarkers = errors.flatMap(i => {
      return [
        {
          severity: monaco.MarkerSeverity.Error,
          message: i.message,
          ...getMarkerSpan(i),
        },
        ...i.hints.map(h => ({
          severity: monaco.MarkerSeverity.Hint,
          message: h,
          ...getMarkerSpan(i),
        }))
      ]
    })

    monaco.editor.setModelMarkers(editor.getModel()!, 'typst', [...warningMarkers, ...errorMarkers])
  }
  
  let editorContainer: HTMLDivElement

  $effect(() => {
    source // is there a better way to tell svelte to react to stuff?
    renderPreview()
  })
  
  let editor: monaco.editor.IStandaloneCodeEditor

  onMount(() => {
    editor = monaco.editor.create(editorContainer, {
      value: defaultSource,
      language: 'typst',
      minimap: {
        enabled: false,
      },
      scrollBeyondLastLine: false,
      overviewRulerLanes: 0,
      wordWrap: 'on',
      theme: 'mathedit',
      fontFamily: 'JetBrains Mono',
      fontWeight: '500',
      fontLigatures: true,
      wordWrapColumn: minWrappingColumns,
    })
    source = editor.getValue()

    editor.onDidChangeModelContent(e => {
      source = editor.getValue()
    })

    registerAutocomplete(webWorld)
  })

  let editorPaneWidth = $state(innerWidth / 2)
  let draggingPaneDivider = false
  let draggingPaneDividerOffset = 0
  let dividerHandle: HTMLDivElement
  let minEditorSize = 200
  // manually measured, ideally there'd be a better way of finding this
  // maybe editor.getLayoutInfo()?
  let minWrappingColumns = 15
  const zoomFitPadding = 10 // px on each side

  $effect(() => {
    editor.updateOptions({ wordWrap: editorPaneWidth < minEditorSize ? 'wordWrapColumn' : 'on' })
  })

  function zoomFit() {
    previewScale = (previewContainer.offsetWidth - zoomFitPadding * 2) / svgWrapper.offsetWidth
    previewTranslation[0] = zoomFitPadding
  }
  function translateTop() {
    previewTranslation[1] = zoomFitPadding
  }


	const appSpinAnimation = new Animation(new KeyframeEffect(document.querySelector('#app'), { rotate: '360deg' }, { duration: 4000, easing: 'cubic-bezier(0.35, 0, 0.2, 1', composite: 'accumulate' }))
</script>

<main>
  <div class="toolbar">
    <div class="toolbar-editor" style:width={`${editorPaneWidth}px`}>
      <button onclick={() => appSpinAnimation.play()}>spin</button>
    </div>
    <div class="toolbar-preview">
      <div class="toolbar-zoom-container">
        <PreviewZoom bind:scale={previewScale} bind:translation={previewTranslation} zoomCenter={previewCenter} />
        <button onclick={zoomFit}>Fit</button>
      </div>
      <span class='zoom-tip'>ctrl+scroll to zoom</span>
    </div>
  </div>
  <div class="panes">
    <div
      bind:this={editorContainer}
      class="editor-container"
      style:width={`${editorPaneWidth}px`}
    ></div>
    <div class="divider">
      <div class="divider-handle"
        bind:this={dividerHandle}
        onpointerdown={e => {
          dividerHandle.setPointerCapture(e.pointerId)
          draggingPaneDividerOffset = editorPaneWidth - e.clientX
          draggingPaneDivider = true
        }}
        onpointermove={e => {
          if (!draggingPaneDivider) return
          editorPaneWidth = e.clientX + draggingPaneDividerOffset
          editor.layout({ width: editorPaneWidth, height: editorContainer.offsetHeight })
          updatePreviewCenter()
          // e.getCoalescedEvents().forEach(e => {}) doesn't seem to help here, which makes sense ig
        }}
        onpointerup={e => {
          dividerHandle.releasePointerCapture(e.pointerId)
          draggingPaneDivider = false
        }}
      ></div>
    </div>
    <div bind:this={previewContainer} class="preview-container">
      <PreviewViewer bind:scale={previewScale} bind:translation={previewTranslation}>
        <div bind:this={svgWrapper} class="svg-wrapper"></div>
      </PreviewViewer>
    </div>
  </div>

</main>


<style>
  .toolbar > * {
    display: flex;
    align-items: center;
    gap: 2rem;
    padding-left: 0.5rem;
  }
  .toolbar-zoom-container {
    display: flex;
    align-items: center;
    gap: 0.3rem;
  }
  .zoom-tip {
    font-size: 0.83rem;
    color: #BBB;
    pointer-events: none;
  }
  main {
    display: flex;
    flex-direction: column;
    height: 100svh;
    overflow: hidden;
  }
  .toolbar {
    padding-block: 0.2rem;
  }
  .toolbar, .panes {
    display: flex;
  }
  .panes {
    flex-grow: 1;
  }
  .divider {
    --width: 1px;
    --padding: 0.3em;
    width: var(--width);
    background: #EEE;
    position: relative;

  }
  .divider-handle {
    position: absolute;
    width: calc(var(--width) + var(--padding) * 2);
    translate: calc(-1 * var(--padding));
    height: 100%;
    cursor: col-resize;
    z-index: 999;
  }

  .preview-container {
		flex: 1 1 0;
		overflow: hidden;
  }

  .svg-wrapper :global(svg) {
    box-shadow: 0 0 11px #00000015;
    border-radius: 0.5rem;
  }
</style>