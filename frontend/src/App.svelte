<script lang="ts">
  import * as wasm from 'mathedit-typst-wasm' // todo: see if top level await on wasm (see npm package) could be removed?
  import * as monaco from 'monaco-editor/esm/vs/editor/editor.api'
  import { onMount } from 'svelte'
  import './lib/userWorker'
  import './lib/monaco-typst-init'
  import { importBundledFonts } from './lib/typst-assets'
  import PreviewViewer from './lib/PreviewViewer.svelte'
  import PreviewZoom from './lib/PreviewZoom.svelte'
  import { addMathDelimiterIndentationListener, registerAutocomplete } from './lib/monaco-typst-init'
  import geode42Logo from './assets/geode42.svg'
  import openInNewTab from './assets/open-in-new-tab.svg?raw'

  const defaultSource = [
    'Some math in Typst',
    '',
    '$(m v^2) / r = q v times B$',
    ''
  ].join('\n')

  const beforeUnloadHandler = (e: BeforeUnloadEvent) => e.preventDefault()

  let source = $state(defaultSource)
  let previewSvgPages: string[] = $state([])
  let previewScale = $state(1)
  let previewTranslation = $state([0, 0])
  let previewContainer: HTMLDivElement
  let svgWrapper: HTMLDivElement
  let logoVisible = $state(true)
  let centeredPageIndex: number | undefined = $state(undefined)

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

  function getCenteredPageIndex() {
    if (previewSvgPages.length == 0) return undefined
    const firstPageRect = (svgWrapper.firstElementChild as SVGElement).getBoundingClientRect()
    const LastPageRect = (svgWrapper.lastElementChild as SVGElement).getBoundingClientRect()
    if (previewCenter[1] < firstPageRect.top) return 0
    if (previewCenter[1] > LastPageRect.bottom) return previewSvgPages.length - 1
    return [...svgWrapper.children].findIndex(page => {
      const rect = (page as SVGElement).getBoundingClientRect()
      return rect.top <= previewCenter[1] && rect.bottom > previewCenter[1]
    })
  }

  $effect(() => {
    // react when these things change
    previewSvgPages
    previewTranslation[0], previewTranslation[1]
    
    centeredPageIndex = getCenteredPageIndex()
  })

  $effect(() => {
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
  const webWorld = new wasm.WebWorld()
  ;(async () => {
    (await importBundledFonts()).forEach(f => webWorld.addFont(f))
    webWorldLoaded = true
    renderPreview()
  })()

  function renderPreview() {
    if (!webWorldLoaded) return
    webWorld.source = source
    webWorld.compile()
    const svgPages = webWorld.toSvgPages()

    if (svgPages) previewSvgPages = svgPages

    const warnings = webWorld.warnings()
    const errors = webWorld.errors()

    // in IMarkerData, `code` and `source` get added in light gray right after the message
    // src: https://code.visualstudio.com/api/references/vscode-api#Diagnostic

    const getMarkerSpan = (diagnostic: wasm.SourceDiagnostic) => {
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
      stickyTabStops: true,
      // keep tabs/spaces synced with `addMathDelimiterIndentationListener` right below !
    })
    source = editor.getValue()

    editor.onDidChangeModelContent(e => {
      source = editor.getValue()
    })

    registerAutocomplete(webWorld)
    addMathDelimiterIndentationListener(editor, '    ')

    editor.onDidContentSizeChange(e => {
      logoVisible = innerHeight - (editorContainer.offsetTop + e.contentHeight) >= minEmptySpaceBelowEditorForLogo
    })
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
  const handleMinDistance = 5 // px on each side, keep synced with `--padding` in CSS below
  const minEmptySpaceBelowEditorForLogo = 70

  function updateLayoutStuff() {
    // clamp divider
    if (editorPaneWidth < handleMinDistance) editorPaneWidth = handleMinDistance
    if (editorPaneWidth > innerWidth - handleMinDistance) editorPaneWidth = innerWidth - handleMinDistance

    editor.layout({ width: editorPaneWidth, height: editorContainer.offsetHeight })
    editor.updateOptions({ wordWrap: editorPaneWidth < minEditorSize ? 'wordWrapColumn' : 'on' })
    updatePreviewCenter()
  }

  addEventListener('resize', updateLayoutStuff)

  // divider location effects
  $effect(() => {
    editorPaneWidth // is there a better way to tell svelte to react to stuff?
    updateLayoutStuff()
  })

  function zoomFit() {
    if (centeredPageIndex == undefined) return

    const centeredPage = [...svgWrapper.children][centeredPageIndex] as SVGElement
    const centerYBeforeTransform = (previewCenter[1] - previewTranslation[1]) / previewScale
    previewScale = (previewContainer.offsetWidth - zoomFitPadding * 2) / centeredPage.clientWidth
    const centerYAfterTransform = (previewCenter[1] - previewTranslation[1]) / previewScale
    
    previewTranslation[0] = zoomFitPadding
    previewTranslation[1] -= (centerYBeforeTransform - centerYAfterTransform) * previewScale // keep center point the same

    // ensure the view is below the top of the first page or above the bottom of the last page
    const maxVerticalTranslation = zoomFitPadding
    const minVerticalTranslation = Math.min(-(svgWrapper.offsetHeight * previewScale) - zoomFitPadding + previewContainer.offsetHeight, maxVerticalTranslation)
    if (previewTranslation[1] > maxVerticalTranslation) previewTranslation[1] = maxVerticalTranslation
    if (previewTranslation[1] < minVerticalTranslation) previewTranslation[1] = minVerticalTranslation
  }
  function translateTop() {
    previewTranslation[1] = zoomFitPadding
  }

  const downloadData = (data: BlobPart[], type = 'text/plain', filename = '') => {
    const url = URL.createObjectURL(new Blob(data, { type }))
    const anchor = document.createElement('a')
    anchor.href = url
    anchor.download = filename
    anchor.click()
    URL.revokeObjectURL(url)
  }

	const appSpinAnimation = new Animation(new KeyframeEffect(document.querySelector('#app'), { rotate: '360deg' }, { duration: 4000, easing: 'cubic-bezier(0.35, 0, 0.2, 1', composite: 'accumulate' }))
</script>

<main>
  <div class="toolbar">
    <div class="toolbar-editor" style:width={`${editorPaneWidth}px`}>
      <button onclick={() => appSpinAnimation.play()}>Spin</button>
      <div class="spacer"></div>
      <a href="https://github.com/geode42/mathedit-typst" target="_blank" rel="noopener noreferrer">GitHub repo</a>
      <div class="spacer"></div>
      <button onclick={() => downloadData([source], 'text/vnd.typst', 'markup.typ')}>Download Source</button>
      <div class="spacer"></div>
      <button onclick={() => downloadData([previewSvgPages[centeredPageIndex!]], 'image/svg+xml', 'output.svg')}>Download SVG</button>
    </div>
    <div class="toolbar-preview">
      <div class="toolbar-zoom-container">
        <PreviewZoom bind:scale={previewScale} bind:translation={previewTranslation} zoomCenter={previewCenter} />
        <button onclick={zoomFit}>Fit</button>
      </div>
      <div class="spacer"></div>
      <span class='zoom-tip'>ctrl+scroll to zoom</span>
      <div class="line-spacer"></div>
      <a class='math-reference-link' href="https://typst.app/docs/reference/math/" target="_blank" rel="noopener noreferrer">Math reference{@html openInNewTab}</a>
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
        <div bind:this={svgWrapper} class="svg-wrapper">
          {#each previewSvgPages as svg}
          {@html svg}
          {/each}
        </div>
      </PreviewViewer>
    </div>
  </div>
  <img class='geode42-logo' class:visible={logoVisible} src={geode42Logo} alt='geode42 logo' />
</main>


<style>
  .toolbar > * {
    display: flex;
    align-items: center;
    /* gap: 2rem; */
    padding-inline: 0.5rem;
    min-width: max-content;

    * {
      width: max-content;
      flex-shrink: 0;
    }

    .spacer {
      flex: 0 1 2rem;
    }
    .line-spacer {
      flex: 1 1 3rem;
    }
  }
  .toolbar-preview {
    flex-grow: 1;
    /* justify-content: end; */
  }
  .toolbar-zoom-container {
    display: flex;
    align-items: center;
    gap: 0.3rem;
  }
  .zoom-tip {
    color: #BBB;
    user-select: none;
  }
  .math-reference-link {
    margin-left: 0.8rem;
    margin-right: 0.1rem;
    :global(svg) {
      height: 0.95em;
      vertical-align: middle;
      margin-left: 0.4em;
    }
  }
  .geode42-logo {
    height: 2.1rem;
    position: absolute;
    bottom: 1.3rem;
    left: 1.3rem;
    opacity: 0;
    transition: opacity 0.5s;
    pointer-events: none;

    &.visible {
      opacity: 0.5;
    }
  }
  main {
    display: flex;
    flex-direction: column;
    height: 100svh;
    overflow: hidden;
  }
  .toolbar {
    padding-block: 0.2rem;

    * {
      font-size: 0.83rem;
    }
  }
  .toolbar, .panes {
    display: flex;
  }
  .panes {
    flex: 1 1 0;
    overflow: hidden;
  }
  .divider {
    --width: 1px;
    --padding: 5px; /* keep synced with `handleMinDistance` in JS above */
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

  .editor-container {
    padding-top: 0.2rem;
  }

  .preview-container {
		flex: 1 1 0;
		overflow: hidden;
  }

  .svg-wrapper {
    display: flex;
    flex-direction: column;
    gap: 1rem;
  }

  .svg-wrapper :global(.typst-doc) {
    box-shadow: 0 0 11px #00000015;
    border-radius: 0.5rem;
  }
</style>