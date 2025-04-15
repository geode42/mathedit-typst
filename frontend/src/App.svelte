<script lang="ts">
  import * as wasm from 'mathedit-typst-wasm' // todo: see if top level await on wasm (see npm package) could be removed?
  import * as monaco from 'monaco-editor/esm/vs/editor/editor.api'
  import { onMount } from 'svelte'
  import './lib/userWorker'
  import './lib/monaco-typst-init'
  import { importBundledFonts } from './lib/typst-assets'
  import PreviewViewer from './lib/PreviewViewer.svelte'

  let source = $state('')
  let previewSvg = $state('')
  let previewScale = $state(1)
  let previewTranslation = $state([0, 0])

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
    previewSvg = webWorld.renderSvg()!
    console.log('hi?', webWorld.errors(), webWorld.warnings())
  }
  
  let editorContainer: HTMLDivElement

  $effect(() => {
    source // is there a better way to tell svelte to react to stuff?
    renderPreview()
  })
  
  let editor: monaco.editor.IStandaloneCodeEditor

  onMount(() => {
    editor = monaco.editor.create(editorContainer, {
      value: 'Some math in Typst\n\n$(m v^2) / r = q v times B$\n',
      language: 'typst',
      minimap: {
        enabled: false,
      },
      scrollBeyondLastLine: false,
      overviewRulerLanes: 0,
      wordWrap: 'on',
      theme: 'mathedit',
      fontFamily: 'JetBrains Mono',
      fontWeight: '450',
      fontLigatures: true,
      // automaticLayout: true,
      wordWrapColumn: minWrappingColumns,
    })
    source = editor.getValue()

    editor.onDidChangeModelContent(e => {
      // console.log(e)
      source = editor.getValue()
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

  $effect(() => {
    editor.updateOptions({ wordWrap: editorPaneWidth < minEditorSize ? 'wordWrapColumn' : 'on' })
  })

	const appSpinAnimation = new Animation(new KeyframeEffect(document.querySelector('#app'), { rotate: '360deg' }, { duration: 4000, easing: 'cubic-bezier(0.35, 0, 0.2, 1', composite: 'accumulate' }))
</script>

<main>
  <div class="toolbar">
    <div class="toolbar-editor" style:width={`${editorPaneWidth}px`}>
      <button onclick={() => appSpinAnimation.play()}>spin</button>
    </div>
    <div class="toolbar-preview">
      <button onclick={() => (previewScale = 1, previewTranslation = [0, 0])}>Reset view</button>
      <span style="font-size: 0.85em; color: #888">ctrl+scroll to zoom</span>
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
          // e.getCoalescedEvents().forEach(e => {}) doesn't seem to help here, which makes sense ig
        }}
        onpointerup={e => {
          dividerHandle.releasePointerCapture(e.pointerId)
          draggingPaneDivider = false
        }}
      ></div>
    </div>
    <PreviewViewer bind:svg={previewSvg} bind:scale={previewScale} bind:translation={previewTranslation} />
  </div>

</main>


<style>
  .toolbar > * {
    display: flex;
    align-items: center;
    gap: 1rem;
  }
  main {
    display: flex;
    flex-direction: column;
    height: 100svh;
    overflow: hidden;
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
  }
</style>