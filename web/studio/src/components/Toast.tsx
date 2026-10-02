import React from 'react'
import { ToastMessage } from '../types/actions'

export interface ToastContainerProps {
  toasts: ToastMessage[]
  onDismiss: (id: string) => void
}

export const ToastContainer: React.FC<ToastContainerProps> = ({ toasts, onDismiss }) => {
  if (toasts.length === 0) return null

  return (
    <div
      style={{
        position: 'fixed',
        bottom: '30px',
        right: '20px',
        display: 'flex',
        flexDirection: 'column',
        gap: '8px',
        zIndex: 2000,
        pointerEvents: 'none',
      }}
    >
      {toasts.map((toast) => {
        const borderAccent =
          toast.type === 'success'
            ? 'var(--accent-green, #22c55e)'
            : toast.type === 'error'
            ? 'var(--accent-rose, #f43f5e)'
            : toast.type === 'warn'
            ? 'var(--accent-amber, #f59e0b)'
            : 'var(--accent-cyan, #38bdf8)'

        return (
          <div
            key={toast.id}
            style={{
              pointerEvents: 'auto',
              minWidth: '280px',
              maxWidth: '380px',
              backgroundColor: 'var(--bg-modal, #0d121f)',
              border: '1px solid var(--border-color, #1a2233)',
              borderLeft: `3px solid ${borderAccent}`,
              borderRadius: '4px',
              boxShadow: '0 8px 24px rgba(0, 0, 0, 0.55)',
              padding: '10px 14px',
              display: 'flex',
              alignItems: 'flex-start',
              justifyContent: 'space-between',
              gap: '10px',
              color: 'var(--text-primary, #f8fafc)',
              animation: 'toastSlideIn 0.2s ease-out',
            }}
          >
            <div style={{ flex: 1 }}>
              <div
                style={{
                  fontSize: 'var(--font-sm, 11px)',
                  fontWeight: 600,
                  marginBottom: toast.message ? '2px' : '0',
                  color: 'var(--text-primary, #f8fafc)',
                }}
              >
                {toast.title}
              </div>
              {toast.message && (
                <div
                  style={{
                    fontSize: 'var(--font-xs, 10px)',
                    color: 'var(--text-muted, #64748b)',
                    lineHeight: '1.4',
                  }}
                >
                  {toast.message}
                </div>
              )}
            </div>

            <button
              onClick={() => onDismiss(toast.id)}
              style={{
                background: 'transparent',
                border: 'none',
                color: 'var(--text-muted, #64748b)',
                cursor: 'pointer',
                padding: '2px',
                display: 'flex',
                alignItems: 'center',
                justifyContent: 'center',
              }}
              onMouseEnter={(e) => (e.currentTarget.style.color = 'var(--text-primary, #f8fafc)')}
              onMouseLeave={(e) => (e.currentTarget.style.color = 'var(--text-muted, #64748b)')}
            >
              <svg width="12" height="12" viewBox="0 0 12 12" fill="none">
                <path
                  d="M2.5 2.5L9.5 9.5M9.5 2.5L2.5 9.5"
                  stroke="currentColor"
                  strokeWidth="1.2"
                  strokeLinecap="round"
                />
              </svg>
            </button>
          </div>
        )
      })}
    </div>
  )
}
