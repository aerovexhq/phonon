import React, { useState, useEffect, useRef } from 'react'
import { ActionDefinition } from '../types/actions'

export interface CommandPaletteProps {
  isOpen: boolean
  onClose: () => void
  actions: ActionDefinition[]
  keybindOverrides: Record<string, string>
}

export const CommandPalette: React.FC<CommandPaletteProps> = ({
  isOpen,
  onClose,
  actions,
  keybindOverrides,
}) => {
  const [query, setQuery] = useState('')
  const [selectedIndex, setSelectedIndex] = useState(0)
  const inputRef = useRef<HTMLInputElement>(null)

  useEffect(() => {
    if (isOpen) {
      setQuery('')
      setSelectedIndex(0)
      setTimeout(() => inputRef.current?.focus(), 50)
    }
  }, [isOpen])

  const filteredActions = actions.filter((action) => {
    const q = query.toLowerCase()
    return (
      action.label.toLowerCase().includes(q) ||
      action.category.toLowerCase().includes(q) ||
      action.description.toLowerCase().includes(q)
    )
  })

  useEffect(() => {
    setSelectedIndex(0)
  }, [query])

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === 'ArrowDown') {
      e.preventDefault()
      setSelectedIndex((prev) => (prev + 1) % Math.max(1, filteredActions.length))
    } else if (e.key === 'ArrowUp') {
      e.preventDefault()
      setSelectedIndex((prev) => (prev - 1 + filteredActions.length) % Math.max(1, filteredActions.length))
    } else if (e.key === 'Enter') {
      e.preventDefault()
      const selected = filteredActions[selectedIndex]
      if (selected) {
        onClose()
        selected.run()
      }
    } else if (e.key === 'Escape') {
      e.preventDefault()
      onClose()
    }
  }

  if (!isOpen) return null

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
        alignItems: 'flex-start',
        justifyContent: 'center',
        paddingTop: '12vh',
        zIndex: 1500,
      }}
      onClick={onClose}
    >
      <div
        style={{
          width: '580px',
          maxWidth: '92vw',
          backgroundColor: 'var(--bg-modal, #0d121f)',
          border: '1px solid var(--border-color, #1a2233)',
          borderRadius: '8px',
          boxShadow: '0 24px 64px rgba(0, 0, 0, 0.75), 0 0 0 1px rgba(255, 255, 255, 0.05)',
          overflow: 'hidden',
          display: 'flex',
          flexDirection: 'column',
          color: 'var(--text-primary, #f8fafc)',
        }}
        onClick={(e) => e.stopPropagation()}
      >
        {/* Search Bar Input */}
        <div
          style={{
            display: 'flex',
            alignItems: 'center',
            padding: '12px 16px',
            borderBottom: '1px solid var(--border-color, #1a2233)',
            backgroundColor: 'var(--bg-header, #090d16)',
            gap: '10px',
          }}
        >
          <svg width="16" height="16" viewBox="0 0 20 20" fill="var(--text-muted, #64748b)">
            <path
              fillRule="evenodd"
              d="M9 3.5a5.5 5.5 0 100 11 5.5 5.5 0 000-11zM2 9a7 7 0 1112.452 4.391l3.328 3.329a.75.75 0 11-1.06 1.06l-3.329-3.328A7 7 0 012 9z"
              clipRule="evenodd"
            />
          </svg>
          <input
            ref={inputRef}
            type="text"
            value={query}
            onChange={(e) => setQuery(e.target.value)}
            onKeyDown={handleKeyDown}
            placeholder="Type a command or search action (e.g. Run DC, Export, Settings)..."
            style={{
              flex: 1,
              backgroundColor: 'transparent',
              border: 'none',
              outline: 'none',
              color: 'var(--text-primary, #f8fafc)',
              fontSize: 'var(--font-lg, 13px)',
              fontFamily: 'inherit',
            }}
          />
          <span
            style={{
              fontSize: 'var(--font-xs, 10px)',
              color: 'var(--text-muted, #64748b)',
              padding: '2px 6px',
              backgroundColor: 'var(--bg-hover, #1e293b)',
              borderRadius: '3px',
              border: '1px solid var(--border-subtle, #131b2c)',
            }}
          >
            ESC to close
          </span>
        </div>

        {/* Action Results List */}
        <div
          style={{
            maxHeight: '380px',
            overflowY: 'auto',
            padding: '6px',
          }}
        >
          {filteredActions.length === 0 ? (
            <div
              style={{
                padding: '24px',
                textAlign: 'center',
                color: 'var(--text-muted, #64748b)',
                fontSize: 'var(--font-sm, 11px)',
              }}
            >
              No matching actions found.
            </div>
          ) : (
            filteredActions.map((action, index) => {
              const isSelected = index === selectedIndex
              const activeKeybind = keybindOverrides[action.id] || action.defaultKeybind

              return (
                <div
                  key={action.id}
                  onClick={() => {
                    onClose()
                    action.run()
                  }}
                  onMouseEnter={() => setSelectedIndex(index)}
                  style={{
                    display: 'flex',
                    alignItems: 'center',
                    justifyContent: 'space-between',
                    padding: '8px 12px',
                    borderRadius: '4px',
                    cursor: 'pointer',
                    backgroundColor: isSelected ? 'var(--bg-hover, #1e293b)' : 'transparent',
                    color: isSelected ? 'var(--accent-cyan, #38bdf8)' : 'var(--text-secondary, #cbd5e1)',
                    transition: 'background-color 0.1s',
                  }}
                >
                  <div style={{ display: 'flex', alignItems: 'center', gap: '8px' }}>
                    <span
                      style={{
                        fontSize: '9px',
                        textTransform: 'uppercase',
                        letterSpacing: '0.04em',
                        padding: '2px 5px',
                        borderRadius: '2px',
                        backgroundColor: 'var(--bg-active, #253349)',
                        color: 'var(--text-muted, #94a3b8)',
                        fontWeight: 600,
                      }}
                    >
                      {action.category}
                    </span>
                    <span
                      style={{
                        fontSize: 'var(--font-sm, 12px)',
                        fontWeight: isSelected ? 600 : 400,
                      }}
                    >
                      {action.label}
                    </span>
                    <span
                      style={{
                        fontSize: 'var(--font-xs, 10px)',
                        color: 'var(--text-muted, #64748b)',
                        marginLeft: '4px',
                      }}
                    >
                      {action.description}
                    </span>
                  </div>

                  {activeKeybind && (
                    <span
                      style={{
                        fontSize: 'var(--font-xs, 10px)',
                        fontFamily: 'monospace',
                        padding: '2px 6px',
                        backgroundColor: 'var(--bg-header, #090d16)',
                        border: '1px solid var(--border-button, #334155)',
                        borderRadius: '3px',
                        color: 'var(--text-muted, #94a3b8)',
                      }}
                    >
                      {activeKeybind}
                    </span>
                  )}
                </div>
              )
            })
          )}
        </div>
      </div>
    </div>
  )
}
