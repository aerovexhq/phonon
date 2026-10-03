import { useEffect, useState } from 'react'

export default function App() {
  const [loading, setLoading] = useState(true)
  const [fadeSplash, setFadeSplash] = useState(false)
  const [error, setError] = useState<string | null>(null)

  useEffect(() => {
    let isMounted = true

    async function initPhononWasm() {
      try {
        const wasmJsPath: string = '/studio/wasm/phonon_gui.js'
        const mod: any = await import(/* @vite-ignore */ wasmJsPath)

        // Initialize WebAssembly binary
        await mod.default('/studio/wasm/phonon_gui_bg.wasm')

        // Start eframe WebRunner bound to canvas #phonon_canvas
        await mod.start('phonon_canvas')

        if (isMounted) {
          setFadeSplash(true)
          setTimeout(() => {
            if (isMounted) setLoading(false)
          }, 350)
        }
      } catch (err: any) {
        console.error('Failed to initialize Phonon WebAssembly Studio Engine:', err)
        if (isMounted) {
          setError(
            err?.message ||
              (typeof err === 'string' ? err : 'Unknown WebAssembly runtime error')
          )
          setLoading(false)
        }
      }
    }

    initPhononWasm()

    return () => {
      isMounted = false
    }
  }, [])

  return (
    <div style={{ position: 'relative', width: '100vw', height: '100vh', overflow: 'hidden' }}>
      {loading && (
        <div className={`wasm-splash-overlay ${fadeSplash ? 'fade-out' : ''}`}>
          <div className="wasm-spinner" />
          <div className="wasm-brand-title">
            <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="#6366f1" strokeWidth="2.5">
              <path d="M12 2v20M17 5H9.5a3.5 3.5 0 0 0 0 7h5a3.5 3.5 0 0 1 0 7H6" strokeLinecap="round" strokeLinejoin="round" />
            </svg>
            Phonon Studio
          </div>
          <div className="wasm-status-text">Booting WebAssembly Desktop Engine...</div>
        </div>
      )}

      {error && (
        <div className="wasm-splash-overlay">
          <div className="wasm-error-card">
            <div className="wasm-error-title">
              <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
                <circle cx="12" cy="12" r="10" />
                <line x1="12" y1="8" x2="12" y2="12" />
                <line x1="12" y1="16" x2="12.01" y2="16" />
              </svg>
              WebAssembly Initialization Failed
            </div>
            <div className="wasm-error-body">
              Phonon Studio requires WebGL 2 and WebAssembly support. If you are developing locally, ensure the WebAssembly engine is compiled using:
              <br />
              <code style={{ color: '#818cf8', display: 'inline-block', marginTop: '6px' }}>
                cargo build --release -p phonon-gui --target wasm32-unknown-unknown
              </code>
            </div>
            <div className="wasm-error-trace">{error}</div>
            <button
              onClick={() => window.location.reload()}
              className="wasm-retry-btn"
            >
              Retry Connection
            </button>
          </div>
        </div>
      )}

      <canvas
        id="phonon_canvas"
        tabIndex={0}
        onContextMenu={(e) => e.preventDefault()}
      />
    </div>
  )
}
