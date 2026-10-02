import React, { useEffect } from 'react'

export interface ConfirmDialogProps {
  isOpen: boolean
  title: string
  message: string
  confirmLabel?: string
  cancelLabel?: string
  discardLabel?: string
  variant?: 'danger' | 'warning' | 'primary'
  onConfirm: () => void
  onCancel: () => void
  onDiscard?: () => void
}

export const ConfirmDialog: React.FC<ConfirmDialogProps> = ({
  isOpen,
  title,
  message,
  confirmLabel = 'Confirm',
  cancelLabel = 'Cancel',
  discardLabel,
  variant = 'warning',
  onConfirm,
  onCancel,
  onDiscard,
}) => {
  useEffect(() => {
    if (!isOpen) return
    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.key === 'Escape') {
        e.stopPropagation()
        onCancel()
      } else if (e.key === 'Enter') {
        e.stopPropagation()
        onConfirm()
      }
    }
    window.addEventListener('keydown', handleKeyDown)
    return () => window.removeEventListener('keydown', handleKeyDown)
  }, [isOpen, onConfirm, onCancel])

  if (!isOpen) return null

  const confirmBg =
    variant === 'danger'
      ? '#e11d48'
      : variant === 'warning'
      ? '#d97706'
      : 'var(--accent-blue, #0284c7)'

  const confirmHoverBg =
    variant === 'danger'
      ? '#be123c'
      : variant === 'warning'
      ? '#b45309'
      : 'var(--accent-cyan, #38bdf8)'

  return (
    <div
      style={{
        position: 'fixed',
        top: 0,
        left: 0,
        right: 0,
        bottom: 0,
        backgroundColor: 'var(--bg-modal-backdrop, rgba(3, 7, 18, 0.75))',
        backdropFilter: 'blur(4px)',
        WebkitBackdropFilter: 'blur(4px)',
        display: 'flex',
        alignItems: 'center',
        justifyContent: 'center',
        zIndex: 1100,
        padding: '20px',
      }}
      onClick={onCancel}
    >
      <div
        style={{
          width: '440px',
          maxWidth: '90vw',
          backgroundColor: 'var(--bg-modal, #0d121f)',
          border: '1px solid var(--border-color, #1a2233)',
          borderRadius: '6px',
          boxShadow: '0 24px 48px rgba(0, 0, 0, 0.75), 0 0 0 1px rgba(255, 255, 255, 0.05)',
          overflow: 'hidden',
          display: 'flex',
          flexDirection: 'column',
          color: 'var(--text-primary, #f8fafc)',
        }}
        onClick={(e) => e.stopPropagation()}
      >
        {/* Header */}
        <div
          style={{
            padding: '14px 18px',
            borderBottom: '1px solid var(--border-subtle, #131b2c)',
            backgroundColor: 'var(--bg-header, #090d16)',
            display: 'flex',
            alignItems: 'center',
            gap: '10px',
          }}
        >
          {variant === 'danger' && (
            <svg width="18" height="18" viewBox="0 0 20 20" fill="#f43f5e">
              <path
                fillRule="evenodd"
                d="M10 18a8 8 0 100-16 8 8 0 000 16zM8.28 7.22a.75.75 0 00-1.06 1.06L8.94 10l-1.72 1.72a.75.75 0 101.06 1.06L10 11.06l1.72 1.72a.75.75 0 101.06-1.06L11.06 10l1.72-1.72a.75.75 0 00-1.06-1.06L10 8.94 8.28 7.22z"
                clipRule="evenodd"
              />
            </svg>
          )}
          {variant === 'warning' && (
            <svg width="18" height="18" viewBox="0 0 20 20" fill="#f59e0b">
              <path
                fillRule="evenodd"
                d="M8.485 2.495c.673-1.167 2.357-1.167 3.03 0l6.28 10.875c.673 1.167-.17 2.625-1.516 2.625H3.72c-1.347 0-2.189-1.458-1.515-2.625L8.485 2.495zM10 5a.75.75 0 01.75.75v3.5a.75.75 0 01-1.5 0v-3.5A.75.75 0 0110 5zm0 9a1 1 0 100-2 1 1 0 000 2z"
                clipRule="evenodd"
              />
            </svg>
          )}
          {variant === 'primary' && (
            <svg width="18" height="18" viewBox="0 0 20 20" fill="#38bdf8">
              <path
                fillRule="evenodd"
                d="M18 10a8 8 0 11-16 0 8 8 0 0116 0zm-7-4a1 1 0 11-2 0 1 1 0 012 0zM9 9a.75.75 0 000 1.5h.253a.25.25 0 01.244.304l-.459 2.066A1.75 1.75 0 0010.747 15H11a.75.75 0 000-1.5h-.253a.25.25 0 01-.244-.304l.459-2.066A1.75 1.75 0 009.253 9H9z"
                clipRule="evenodd"
              />
            </svg>
          )}
          <span style={{ fontSize: 'var(--font-title, 14px)', fontWeight: 600 }}>
            {title}
          </span>
        </div>

        {/* Message */}
        <div style={{ padding: '18px', fontSize: 'var(--font-sm, 12px)', color: 'var(--text-secondary, #cbd5e1)', lineHeight: '1.5' }}>
          {message}
        </div>

        {/* Action Buttons */}
        <div
          style={{
            padding: '12px 18px',
            backgroundColor: 'var(--bg-header, #090d16)',
            borderTop: '1px solid var(--border-subtle, #131b2c)',
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'flex-end',
            gap: '8px',
          }}
        >
          <button
            onClick={onCancel}
            style={{
              padding: '6px 14px',
              fontSize: 'var(--font-sm, 11px)',
              fontWeight: 500,
              backgroundColor: 'var(--bg-hover, #1e293b)',
              color: 'var(--text-secondary, #cbd5e1)',
              border: '1px solid var(--border-button, #334155)',
              borderRadius: '4px',
              cursor: 'pointer',
            }}
            onMouseEnter={(e) => {
              e.currentTarget.style.backgroundColor = 'var(--bg-active, #253349)'
              e.currentTarget.style.color = 'var(--text-primary, #f8fafc)'
            }}
            onMouseLeave={(e) => {
              e.currentTarget.style.backgroundColor = 'var(--bg-hover, #1e293b)'
              e.currentTarget.style.color = 'var(--text-secondary, #cbd5e1)'
            }}
          >
            {cancelLabel}
          </button>

          {discardLabel && onDiscard && (
            <button
              onClick={onDiscard}
              style={{
                padding: '6px 14px',
                fontSize: 'var(--font-sm, 11px)',
                fontWeight: 500,
                backgroundColor: 'transparent',
                color: '#f43f5e',
                border: '1px solid rgba(244, 63, 94, 0.4)',
                borderRadius: '4px',
                cursor: 'pointer',
              }}
              onMouseEnter={(e) => {
                e.currentTarget.style.backgroundColor = 'rgba(244, 63, 94, 0.15)'
              }}
              onMouseLeave={(e) => {
                e.currentTarget.style.backgroundColor = 'transparent'
              }}
            >
              {discardLabel}
            </button>
          )}

          <button
            onClick={onConfirm}
            style={{
              padding: '6px 14px',
              fontSize: 'var(--font-sm, 11px)',
              fontWeight: 600,
              backgroundColor: confirmBg,
              color: '#ffffff',
              border: 'none',
              borderRadius: '4px',
              cursor: 'pointer',
              transition: 'background-color 0.15s',
            }}
            onMouseEnter={(e) => (e.currentTarget.style.backgroundColor = confirmHoverBg)}
            onMouseLeave={(e) => (e.currentTarget.style.backgroundColor = confirmBg)}
          >
            {confirmLabel}
          </button>
        </div>
      </div>
    </div>
  )
}
