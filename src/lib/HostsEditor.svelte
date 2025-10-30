<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { EditorView, basicSetup } from 'codemirror';
  import { EditorState } from '@codemirror/state';
  import { LanguageSupport } from '@codemirror/language';
  import { searchKeymap, highlightSelectionMatches } from '@codemirror/search';
  import { autocompletion } from '@codemirror/autocomplete';
  import { indentOnInput } from '@codemirror/language';
  import { lineNumbers, highlightActiveLineGutter, keymap } from '@codemirror/view';
  import { search, closeSearchPanel } from '@codemirror/search';

  interface Props {
    value?: string;
    readonly?: boolean;
    onchange?: (value: string) => void;
  }

  let { 
    value = '', 
    readonly = false,
    onchange
  }: Props = $props();

  let editorValue = $state(value);
  let editorReadonly = $state(readonly);

  let editorContainer: HTMLDivElement;
  let view: EditorView;
  let searchOpen = $state(false);

  // Common hosts file completions
  const hostsCompletions = (context: any) => {
    const word = context.matchBefore(/\w*/);
    if (!word || (word.from === word.to && !context.explicit)) {
      return null;
    }
    
    return {
      from: word.from,
      options: [
        { label: 'localhost', type: 'constant' },
        { label: '127.0.0.1', type: 'number' },
        { label: '0.0.0.0', type: 'number' },
        { label: '::1', type: 'number' },
        { label: 'broadcasthost', type: 'constant' },
        { label: '0.255.255.255', type: 'number' }
      ]
    };
  };

  const autocompletionConfig = {
    override: [hostsCompletions]
  };

  onMount(() => {
    console.log("HostsEditor mounting with value:", value?.length || 0, "characters");
    console.log("First 100 chars:", value?.substring(0, 100));
    
    // Remove BOM (Byte Order Mark) if present
    const cleanValue = (value || editorValue).replace(/^\uFEFF/, '');
    
    const startState = EditorState.create({
      doc: cleanValue,
      extensions: [
        basicSetup,
        lineNumbers(),
        highlightActiveLineGutter(),
        EditorView.lineWrapping,
        EditorState.readOnly.of(editorReadonly),
        autocompletion(autocompletionConfig),
        highlightSelectionMatches(),
        indentOnInput(),
        search({ top: true }),
        EditorView.updateListener.of((update) => {
          if (update.docChanged && !editorReadonly) {
            const newContent = update.state.doc.toString();
            editorValue = newContent;
            onchange?.(newContent);
          }
        }),
        keymap.of([
          { key: "Mod-f", run: toggleSearch }
        ]),
        EditorView.theme({
          '&': {
            height: '100%',
            backgroundColor: '#ffffff',
            display: 'flex',
            flexDirection: 'column'
          },
          '.cm-scroller': {
            overflow: 'auto',
            fontFamily: 'ui-monospace, SFMono-Regular, "SF Mono", Consolas, "Liberation Mono", Menlo, monospace',
            fontSize: '14px',
            lineHeight: '1.5',
            flex: '1'
          },
          '.cm-editor': {
            height: '100%',
            display: 'flex',
            flexDirection: 'column'
          },
          '.cm-focused': {
            outline: '2px solid #3b82f6',
            outlineOffset: '-2px'
          },
          '.cm-content': {
            padding: '12px',
            minHeight: '100%',
            boxSizing: 'border-box'
          },
          '.cm-line': {
            padding: '0 0 0.1em 0'
          },
          '.cm-comment': {
            color: '#6b7280',
            fontStyle: 'italic'
          },
          '.cm-number': {
            color: '#dc2626',
            fontWeight: 'bold'
          },
          '.cm-variableName': {
            color: '#1f2937'
          },
          '.cm-atom': {
            color: '#059669',
            fontWeight: 'bold'
          },
          '.cm-keyword': {
            color: '#7c3aed'
          },
          '.cm-selectionBackground, ::selection': {
            backgroundColor: '#dbeafe'
          },
          '.cm-gutters': {
            backgroundColor: '#f9fafb',
            borderRight: '1px solid #e5e7eb'
          },
          '.cm-activeLineGutter': {
            backgroundColor: '#f3f4f6'
          },
          '.cm-lineNumbers .cm-gutterElement': {
            color: '#6b7280',
            padding: '0 8px 0 16px',
            minWidth: '40px',
            textAlign: 'right'
          }
        })
      ]
    });

    view = new EditorView({
      state: startState,
      parent: editorContainer
    });

    // Force a refresh after a short delay to ensure proper rendering
    setTimeout(() => {
      if (view) {
        view.requestMeasure();
        console.log("Editor initialized with content:", view.state.doc.toString().length, "characters");
      }
    }, 50);
  });

  onDestroy(() => {
    if (view) {
      view.destroy();
    }
  });

  // Update editor when value changes externally
  $effect(() => {
    if (view && editorValue !== view.state.doc.toString()) {
      view.dispatch({
        changes: {
          from: 0,
          to: view.state.doc.length,
          insert: editorValue || ''
        }
      });
    }
  });

  // Update readonly state
  $effect(() => {
    if (view && editorReadonly !== view.state.readOnly) {
      view.dispatch({
        effects: EditorState.readOnly.reconfigure(EditorState.readOnly.of(editorReadonly))
      });
    }
  });

  // Sync props with state and update editor
  $effect(() => {
    // Remove BOM (Byte Order Mark) if present
    const newValue = (value || "").replace(/^\uFEFF/, '');
    editorValue = newValue;
    
    // Update editor document if view exists
    if (view) {
      const currentContent = view.state.doc.toString();
      if (newValue !== currentContent) {
        console.log("Updating editor content:", newValue.length, "characters");
        view.dispatch({
          changes: {
            from: 0,
            to: view.state.doc.length,
            insert: newValue
          }
        });
      }
    }
  });
  
  $effect(() => {
    editorReadonly = readonly;
  });

  function toggleSearch() {
    if (view) {
      // Check if search panel is already open
      const hasSearchPanel = view.dom.querySelector('.cm-search');
      if (hasSearchPanel) {
        // Close search panel
        view.dispatch({ effects: closeSearchPanel.of(null) });
      } else {
        // Open search panel - trigger the search command
        view.focus();
        // Simulate Ctrl+F keypress
        const event = new KeyboardEvent('keydown', {
          key: 'f',
          ctrlKey: true,
          bubbles: true
        });
        view.contentDOM.dispatchEvent(event);
      }
    }
  }

  function copyContent() {
    if (view) {
      navigator.clipboard.writeText(view.state.doc.toString());
    }
  }
</script>

<div class="hosts-editor-wrapper w-full h-full flex flex-col">
  <!-- Editor Toolbar -->
  <div class="editor-toolbar flex items-center gap-2 p-2 bg-gray-50 border-b border-gray-200">
    <button 
      type="button"
      class="btn btn-xs btn-ghost" 
      onclick={toggleSearch}
      title="Search (Ctrl+F) - Click to toggle search panel"
    >
      <svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z" />
      </svg>
      Search
    </button>
    
    <button 
      type="button"
      class="btn btn-xs btn-ghost" 
      onclick={copyContent}
      title="Copy to clipboard"
    >
      <svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8 16H6a2 2 0 01-2-2V6a2 2 0 012-2h8a2 2 0 012 2v2m-6 12h8a2 2 0 002-2v-8a2 2 0 00-2-2h-8a2 2 0 00-2 2v8a2 2 0 002 2z" />
      </svg>
      Copy
    </button>
    
    <div class="flex-1"></div>
    
    <div class="text-xs text-gray-500">
      {editorValue.split('\n').length} lines • {editorValue.length} chars
    </div>
  </div>
  
  <!-- Editor Container -->
  <div bind:this={editorContainer} class="hosts-editor flex-1"></div>
</div>

<style>
  .hosts-editor-wrapper {
    border: 1px solid #d1d5db;
    border-radius: 0.375rem;
    background-color: white;
    overflow: hidden;
    height: 100%;
    display: flex;
    flex-direction: column;
  }
  
  .hosts-editor {
    border: none;
    border-radius: 0;
    background-color: white;
    overflow: hidden;
    flex: 1;
    min-height: 0;
    display: flex;
    flex-direction: column;
  }
  
  .hosts-editor :global(.cm-editor) {
    height: 100%;
    flex: 1;
    display: flex;
    flex-direction: column;
  }
  
  .hosts-editor :global(.cm-scroller) {
    overflow: auto !important;
    scrollbar-width: thin;
    scrollbar-color: #d1d5db #f9fafb;
    flex: 1;
    contain: strict;
    min-height: 200px;
    max-height: 100%;
  }
  
  .hosts-editor :global(.cm-scroller::-webkit-scrollbar) {
    width: 12px;
    height: 12px;
  }
  
  .hosts-editor :global(.cm-scroller::-webkit-scrollbar-track) {
    background: #f9fafb;
  }
  
  .hosts-editor :global(.cm-scroller::-webkit-scrollbar-thumb) {
    background: #d1d5db;
    border-radius: 6px;
    border: 2px solid #f9fafb;
  }
  
  .hosts-editor :global(.cm-scroller::-webkit-scrollbar-thumb:hover) {
    background: #9ca3af;
  }
  
  .hosts-editor :global(.cm-content) {
    padding: 12px;
    min-height: 100%;
    box-sizing: border-box;
  }
  
  .hosts-editor :global(.cm-line) {
    padding: 0 0 0.1em 0;
  }
  
  .editor-toolbar {
    flex-shrink: 0;
  }
  
  .editor-toolbar :global(.btn:disabled) {
    opacity: 0.5;
    cursor: not-allowed;
  }
</style>
