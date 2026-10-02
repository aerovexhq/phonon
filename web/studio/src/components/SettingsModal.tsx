import React, { useState } from 'react'
import { Modal } from './Modal'
import { ActionDefinition, CustomTheme, FontSizeOption, StudioSettings } from '../types/actions'

export interface SettingsModalProps {
  isOpen: boolean
  onClose: () => void
  settings: StudioSettings
  onUpdateSettings: (newSettings: Partial<StudioSettings>) => void
  actions: ActionDefinition[]
  onShowToast: (title: string, message?: string, type?: 'success' | 'info' | 'warn' | 'error') => void
}

type SettingsTab = 'general' | 'appearance' | 'keybindings' | 'simulation'

export const SettingsModal: React.FC<SettingsModalProps> = ({
  isOpen,
  onClose,
  settings,
  onUpdateSettings,
  actions,
  onShowToast,
}) => {
  const [activeTab, setActiveTab] = useState<SettingsTab>('general')
  const [keybindSearch, setKeybindSearch] = useState('')
  const [recordingActionId, setRecordingActionId] = useState<string | null>(null)

  // Custom theme editor state
  const [newThemeName, setNewThemeName] = useState('')
  const [customThemeColors, setCustomThemeColors] = useState({
    '--bg-canvas': '#0b0f19',
    '--bg-header': '#090d16',
    '--bg-panel': '#0d111a',
    '--border-color': '#1a2233',
    '--text-primary': '#f8fafc',
    '--canvas-wire': '#22c55e',
    '--accent-cyan': '#38bdf8',
  })

  // Key recording listener
  React.useEffect(() => {
    if (!recordingActionId) return

    const handleKeyRecord = (e: KeyboardEvent) => {
      e.preventDefault()
      e.stopPropagation()

      if (e.key === 'Escape') {
        setRecordingActionId(null)
        return
      }

      // Build key combination
      const parts: string[] = []
      if (e.ctrlKey || e.metaKey) parts.push('Ctrl')
      if (e.altKey) parts.push('Alt')
      if (e.shiftKey) parts.push('Shift')

      const keyName = e.key.length === 1 ? e.key.toUpperCase() : e.key
      if (!['Control', 'Shift', 'Alt', 'Meta'].includes(keyName)) {
        parts.push(keyName)
      }

      if (parts.length > 0 && !['Control', 'Shift', 'Alt', 'Meta'].includes(keyName)) {
        const combo = parts.join('+')
        const updated = { ...settings.keybindOverrides, [recordingActionId]: combo }
        onUpdateSettings({ keybindOverrides: updated })
        onShowToast('Keybind Updated', `Assigned ${combo} to action`, 'success')
        setRecordingActionId(null)
      }
    }

    window.addEventListener('keydown', handleKeyRecord, true)
    return () => window.removeEventListener('keydown', handleKeyRecord, true)
  }, [recordingActionId, settings.keybindOverrides, onUpdateSettings, onShowToast])

  const handleCreateCustomTheme = () => {
    if (!newThemeName.trim()) {
      onShowToast('Validation Error', 'Please enter a name for the custom theme', 'warn')
      return
    }

    const themeId = `custom-${Date.now()}`
    const newTheme: CustomTheme = {
      id: themeId,
      name: newThemeName.trim(),
      variables: { ...customThemeColors },
    }

    const updatedThemes = [...settings.customThemes, newTheme]
    onUpdateSettings({
      customThemes: updatedThemes,
      theme: themeId,
    })
    setNewThemeName('')
    onShowToast('Theme Created', `Custom theme "${newTheme.name}" created and applied`, 'success')
  }

  const handleDeleteCustomTheme = (themeId: string) => {
    const updated = settings.customThemes.filter((t) => t.id !== themeId)
    const fallbackTheme = settings.theme === themeId ? 'theme-deep-space' : settings.theme
    onUpdateSettings({ customThemes: updated, theme: fallbackTheme })
    onShowToast('Theme Removed', 'Custom theme deleted', 'info')
  }

  const handleClearKeybind = (actionId: string) => {
    const updated = { ...settings.keybindOverrides }
    delete updated[actionId]
    onUpdateSettings({ keybindOverrides: updated })
    onShowToast('Keybind Cleared', 'Reset keybind for action', 'info')
  }

  const handleResetAllKeybinds = () => {
    onUpdateSettings({ keybindOverrides: {} })
    onShowToast('Keybindings Reset', 'All keybindings restored to defaults', 'success')
  }

  const filteredKeybindActions = actions.filter((action) => {
    const q = keybindSearch.toLowerCase()
    return (
      action.label.toLowerCase().includes(q) ||
      action.category.toLowerCase().includes(q) ||
      action.description.toLowerCase().includes(q)
    )
  })

  return (
    <Modal
      isOpen={isOpen}
      onClose={onClose}
      title="Studio Preferences & Configuration"
      width="72vw"
      height="88vh"
      maxWidth="1100px"
      maxHeight="92vh"
      footer={
        <button
          onClick={onClose}
          style={{
            padding: '6px 16px',
            fontSize: 'var(--font-sm, 11px)',
            fontWeight: 600,
            backgroundColor: 'var(--accent-blue, #0284c7)',
            color: '#ffffff',
            border: 'none',
            borderRadius: '4px',
            cursor: 'pointer',
          }}
          onMouseEnter={(e) => (e.currentTarget.style.backgroundColor = 'var(--accent-cyan, #38bdf8)')}
          onMouseLeave={(e) => (e.currentTarget.style.backgroundColor = 'var(--accent-blue, #0284c7)')}
        >
          Done
        </button>
      }
    >
      <div style={{ display: 'flex', flex: 1, overflow: 'hidden' }}>
        {/* Left Sidebar Tabs */}
        <div
          style={{
            width: '200px',
            backgroundColor: 'var(--bg-panel-subtle, #090d16)',
            borderRight: '1px solid var(--border-color, #1a2233)',
            display: 'flex',
            flexDirection: 'column',
            padding: '12px 8px',
            gap: '2px',
            flexShrink: 0,
          }}
        >
          {[
            { id: 'general', label: 'General Preferences' },
            { id: 'appearance', label: 'Appearance & Themes' },
            { id: 'keybindings', label: 'Keybinding Registry' },
            { id: 'simulation', label: 'Simulation Engine' },
          ].map((tab) => {
            const isActive = activeTab === tab.id
            return (
              <div
                key={tab.id}
                onClick={() => setActiveTab(tab.id as SettingsTab)}
                style={{
                  padding: '8px 12px',
                  borderRadius: '4px',
                  cursor: 'pointer',
                  fontSize: 'var(--font-sm, 11.5px)',
                  fontWeight: isActive ? 600 : 500,
                  backgroundColor: isActive ? 'var(--bg-hover, #1e293b)' : 'transparent',
                  color: isActive ? 'var(--accent-cyan, #38bdf8)' : 'var(--text-secondary, #cbd5e1)',
                  transition: 'background-color 0.15s, color 0.15s',
                }}
              >
                {tab.label}
              </div>
            )
          })}
        </div>

        {/* Right Content View */}
        <div
          style={{
            flex: 1,
            overflowY: 'auto',
            padding: '24px 28px',
            backgroundColor: 'var(--bg-panel, #0d111a)',
          }}
        >
          {/* TAB 1: GENERAL */}
          {activeTab === 'general' && (
            <div>
              <div style={{ fontSize: 'var(--font-title, 15px)', fontWeight: 600, marginBottom: '4px' }}>
                General Preferences
              </div>
              <div style={{ fontSize: 'var(--font-xs, 11px)', color: 'var(--text-muted, #64748b)', marginBottom: '20px' }}>
                Configure global UI font size scaling, grid behavior, and canvas snapping.
              </div>

              {/* Font Size Selector */}
              <div style={{ marginBottom: '24px' }}>
                <label style={{ display: 'block', fontSize: 'var(--font-sm, 12px)', fontWeight: 600, marginBottom: '8px' }}>
                  UI & Canvas Font Size
                </label>
                <div style={{ display: 'flex', gap: '8px' }}>
                  {[
                    { id: 'sm', label: 'Small (10px)', desc: 'Compact high-density CAD display' },
                    { id: 'md', label: 'Medium (12px)', desc: 'Standard default studio scaling' },
                    { id: 'lg', label: 'Large (13.5px)', desc: 'Comfortable presentation readability' },
                    { id: 'xl', label: 'Extra Large (15px)', desc: 'Maximum accessibility scaling' },
                  ].map((size) => {
                    const isSelected = settings.fontSize === size.id
                    return (
                      <div
                        key={size.id}
                        onClick={() => {
                          onUpdateSettings({ fontSize: size.id as FontSizeOption })
                          onShowToast('Font Size Updated', `Switched font scale to ${size.label}`, 'info')
                        }}
                        style={{
                          flex: 1,
                          padding: '10px 12px',
                          borderRadius: '4px',
                          border: `1px solid ${isSelected ? 'var(--accent-cyan, #38bdf8)' : 'var(--border-button, #334155)'}`,
                          backgroundColor: isSelected ? 'var(--bg-active, #253349)' : 'var(--bg-input, #131b2c)',
                          cursor: 'pointer',
                          transition: 'border-color 0.15s',
                        }}
                      >
                        <div style={{ fontSize: 'var(--font-sm, 12px)', fontWeight: 600, color: isSelected ? 'var(--accent-cyan, #38bdf8)' : 'var(--text-primary, #f8fafc)' }}>
                          {size.label}
                        </div>
                        <div style={{ fontSize: '10px', color: 'var(--text-muted, #64748b)', marginTop: '2px' }}>
                          {size.desc}
                        </div>
                      </div>
                    )
                  })}
                </div>
              </div>

              {/* Canvas Snapping & Grid Options */}
              <div style={{ marginBottom: '24px' }}>
                <label style={{ display: 'block', fontSize: 'var(--font-sm, 12px)', fontWeight: 600, marginBottom: '8px' }}>
                  Canvas & Routing Options
                </label>
                <div style={{ display: 'flex', flexDirection: 'column', gap: '10px' }}>
                  <label style={{ display: 'flex', alignItems: 'center', gap: '8px', cursor: 'pointer', fontSize: 'var(--font-sm, 12px)' }}>
                    <input
                      type="checkbox"
                      checked={settings.gridSnap}
                      onChange={(e) => onUpdateSettings({ gridSnap: e.target.checked })}
                      style={{ cursor: 'pointer' }}
                    />
                    Enable 10px Orthogonal Grid Snapping
                  </label>
                  <label style={{ display: 'flex', alignItems: 'center', gap: '8px', cursor: 'pointer', fontSize: 'var(--font-sm, 12px)' }}>
                    <input
                      type="checkbox"
                      checked={settings.showGrid}
                      onChange={(e) => onUpdateSettings({ showGrid: e.target.checked })}
                      style={{ cursor: 'pointer' }}
                    />
                    Render 20px Visual Dot Grid Background
                  </label>
                </div>
              </div>
            </div>
          )}

          {/* TAB 2: APPEARANCE & THEMES */}
          {activeTab === 'appearance' && (
            <div>
              <div style={{ fontSize: 'var(--font-title, 15px)', fontWeight: 600, marginBottom: '4px' }}>
                Appearance & Central Theme Engine
              </div>
              <div style={{ fontSize: 'var(--font-xs, 11px)', color: 'var(--text-muted, #64748b)', marginBottom: '20px' }}>
                Select an engineering color palette or customize CSS variables to create your own bespoke theme.
              </div>

              {/* Built-in Themes Grid */}
              <div style={{ marginBottom: '24px' }}>
                <label style={{ display: 'block', fontSize: 'var(--font-sm, 12px)', fontWeight: 600, marginBottom: '10px' }}>
                  Built-in Themes
                </label>
                <div style={{ display: 'grid', gridTemplateColumns: 'repeat(3, 1fr)', gap: '10px' }}>
                  {[
                    { id: 'theme-deep-space', name: 'Deep Space (Default)', bg: '#0b0f19', border: '#1a2233', accent: '#38bdf8' },
                    { id: 'theme-obsidian', name: 'OLED Obsidian', bg: '#000000', border: '#202020', accent: '#00e5ff' },
                    { id: 'theme-nord', name: 'Nordic Polar', bg: '#242933', border: '#3b4252', accent: '#88c0d0' },
                    { id: 'theme-monokai', name: 'Monokai Pro', bg: '#221f22', border: '#383539', accent: '#a9dc76' },
                    { id: 'theme-solarized', name: 'Solarized Dark', bg: '#002b36', border: '#0e5668', accent: '#2aa198' },
                    { id: 'theme-light-lab', name: 'Clean Scientific Light', bg: '#f8fafc', border: '#cbd5e1', accent: '#0284c7' },
                  ].map((t) => {
                    const isSelected = settings.theme === t.id
                    return (
                      <div
                        key={t.id}
                        onClick={() => {
                          onUpdateSettings({ theme: t.id })
                          onShowToast('Theme Applied', `Switched theme to ${t.name}`, 'success')
                        }}
                        style={{
                          padding: '10px',
                          borderRadius: '4px',
                          border: `1px solid ${isSelected ? 'var(--accent-cyan, #38bdf8)' : 'var(--border-button, #334155)'}`,
                          backgroundColor: 'var(--bg-input, #131b2c)',
                          cursor: 'pointer',
                          display: 'flex',
                          alignItems: 'center',
                          gap: '10px',
                        }}
                      >
                        <div
                          style={{
                            width: '24px',
                            height: '24px',
                            borderRadius: '3px',
                            backgroundColor: t.bg,
                            border: `2px solid ${t.accent}`,
                            flexShrink: 0,
                          }}
                        />
                        <div style={{ flex: 1 }}>
                          <div style={{ fontSize: 'var(--font-sm, 11px)', fontWeight: isSelected ? 600 : 500, color: isSelected ? 'var(--accent-cyan, #38bdf8)' : 'var(--text-primary, #f8fafc)' }}>
                            {t.name}
                          </div>
                        </div>
                      </div>
                    )
                  })}
                </div>
              </div>

              {/* Custom Themes List */}
              {settings.customThemes.length > 0 && (
                <div style={{ marginBottom: '24px' }}>
                  <label style={{ display: 'block', fontSize: 'var(--font-sm, 12px)', fontWeight: 600, marginBottom: '8px' }}>
                    Custom User Themes
                  </label>
                  <div style={{ display: 'flex', flexDirection: 'column', gap: '6px' }}>
                    {settings.customThemes.map((ct) => {
                      const isSelected = settings.theme === ct.id
                      return (
                        <div
                          key={ct.id}
                          style={{
                            display: 'flex',
                            alignItems: 'center',
                            justifyContent: 'space-between',
                            padding: '8px 12px',
                            backgroundColor: 'var(--bg-input, #131b2c)',
                            border: `1px solid ${isSelected ? 'var(--accent-cyan, #38bdf8)' : 'var(--border-subtle, #131b2c)'}`,
                            borderRadius: '4px',
                          }}
                        >
                          <span style={{ fontSize: 'var(--font-sm, 12px)', fontWeight: 600, color: isSelected ? 'var(--accent-cyan, #38bdf8)' : 'var(--text-primary, #f8fafc)' }}>
                            {ct.name}
                          </span>
                          <div style={{ display: 'flex', gap: '6px' }}>
                            <button
                              onClick={() => {
                                onUpdateSettings({ theme: ct.id })
                                onShowToast('Theme Applied', `Applied custom theme "${ct.name}"`, 'success')
                              }}
                              style={{
                                padding: '3px 8px',
                                fontSize: '10px',
                                backgroundColor: isSelected ? 'var(--accent-blue, #0284c7)' : 'var(--bg-hover, #1e293b)',
                                color: '#ffffff',
                                border: '1px solid var(--border-button, #334155)',
                                borderRadius: '3px',
                                cursor: 'pointer',
                              }}
                            >
                              {isSelected ? 'Active' : 'Apply'}
                            </button>
                            <button
                              onClick={() => handleDeleteCustomTheme(ct.id)}
                              style={{
                                padding: '3px 8px',
                                fontSize: '10px',
                                backgroundColor: 'transparent',
                                color: '#f43f5e',
                                border: '1px solid rgba(244, 63, 94, 0.4)',
                                borderRadius: '3px',
                                cursor: 'pointer',
                              }}
                            >
                              Delete
                            </button>
                          </div>
                        </div>
                      )
                    })}
                  </div>
                </div>
              )}

              {/* Create Custom Theme Section */}
              <div
                style={{
                  padding: '14px',
                  backgroundColor: 'var(--bg-input, #131b2c)',
                  border: '1px solid var(--border-subtle, #131b2c)',
                  borderRadius: '6px',
                }}
              >
                <div style={{ fontSize: 'var(--font-sm, 12px)', fontWeight: 600, marginBottom: '8px' }}>
                  Create New Custom Theme
                </div>
                <div style={{ display: 'flex', gap: '8px', marginBottom: '12px' }}>
                  <input
                    type="text"
                    value={newThemeName}
                    onChange={(e) => setNewThemeName(e.target.value)}
                    placeholder="Custom theme name (e.g. Cyberpunk Neon)..."
                    style={{
                      flex: 1,
                      padding: '6px 10px',
                      backgroundColor: 'var(--bg-modal, #0d121f)',
                      border: '1px solid var(--border-button, #334155)',
                      borderRadius: '3px',
                      color: 'var(--text-primary, #f8fafc)',
                      fontSize: 'var(--font-sm, 11px)',
                    }}
                  />
                  <button
                    onClick={handleCreateCustomTheme}
                    style={{
                      padding: '6px 14px',
                      fontSize: 'var(--font-sm, 11px)',
                      fontWeight: 600,
                      backgroundColor: 'var(--accent-blue, #0284c7)',
                      color: '#ffffff',
                      border: 'none',
                      borderRadius: '3px',
                      cursor: 'pointer',
                    }}
                  >
                    Save & Apply
                  </button>
                </div>

                {/* Color Variable Pickers */}
                <div style={{ display: 'grid', gridTemplateColumns: 'repeat(2, 1fr)', gap: '8px' }}>
                  {[
                    { key: '--bg-canvas', label: 'Canvas Background' },
                    { key: '--bg-header', label: 'Header & Toolbar' },
                    { key: '--bg-panel', label: 'Sidebar Panels' },
                    { key: '--text-primary', label: 'Primary Text' },
                    { key: '--canvas-wire', label: 'Wire Trace Color' },
                    { key: '--accent-cyan', label: 'Primary Accent' },
                  ].map((field) => (
                    <div
                      key={field.key}
                      style={{
                        display: 'flex',
                        alignItems: 'center',
                        justifyContent: 'space-between',
                        padding: '4px 8px',
                        backgroundColor: 'var(--bg-modal, #0d121f)',
                        borderRadius: '3px',
                      }}
                    >
                      <span style={{ fontSize: '10px', color: 'var(--text-secondary, #cbd5e1)' }}>
                        {field.label}
                      </span>
                      <input
                        type="color"
                        value={customThemeColors[field.key as keyof typeof customThemeColors] || '#000000'}
                        onChange={(e) =>
                          setCustomThemeColors((prev) => ({
                            ...prev,
                            [field.key]: e.target.value,
                          }))
                        }
                        style={{
                          width: '24px',
                          height: '20px',
                          border: 'none',
                          cursor: 'pointer',
                          backgroundColor: 'transparent',
                        }}
                      />
                    </div>
                  ))}
                </div>
              </div>
            </div>
          )}

          {/* TAB 3: KEYBINDINGS */}
          {activeTab === 'keybindings' && (
            <div>
              <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', marginBottom: '8px' }}>
                <div>
                  <div style={{ fontSize: 'var(--font-title, 15px)', fontWeight: 600 }}>
                    Keybinding Action Registry
                  </div>
                  <div style={{ fontSize: 'var(--font-xs, 11px)', color: 'var(--text-muted, #64748b)' }}>
                    Customize, assign, or remove keyboard shortcuts for any action in the studio.
                  </div>
                </div>
                <button
                  onClick={handleResetAllKeybinds}
                  style={{
                    padding: '4px 10px',
                    fontSize: '10px',
                    backgroundColor: 'var(--bg-hover, #1e293b)',
                    color: 'var(--text-muted, #94a3b8)',
                    border: '1px solid var(--border-button, #334155)',
                    borderRadius: '3px',
                    cursor: 'pointer',
                  }}
                >
                  Reset Defaults
                </button>
              </div>

              {/* Search Filter */}
              <div style={{ marginBottom: '14px' }}>
                <input
                  type="text"
                  value={keybindSearch}
                  onChange={(e) => setKeybindSearch(e.target.value)}
                  placeholder="Filter keybindings by action or category..."
                  style={{
                    width: '100%',
                    padding: '6px 10px',
                    backgroundColor: 'var(--bg-input, #131b2c)',
                    border: '1px solid var(--border-button, #334155)',
                    borderRadius: '4px',
                    color: 'var(--text-primary, #f8fafc)',
                    fontSize: 'var(--font-sm, 11px)',
                    outline: 'none',
                  }}
                />
              </div>

              {/* Keybindings Table */}
              <div
                style={{
                  maxHeight: '440px',
                  overflowY: 'auto',
                  border: '1px solid var(--border-color, #1a2233)',
                  borderRadius: '4px',
                }}
              >
                {filteredKeybindActions.map((action) => {
                  const currentCombo = settings.keybindOverrides[action.id] || action.defaultKeybind
                  const isRecording = recordingActionId === action.id

                  return (
                    <div
                      key={action.id}
                      style={{
                        display: 'flex',
                        alignItems: 'center',
                        justifyContent: 'space-between',
                        padding: '8px 12px',
                        borderBottom: '1px solid var(--border-subtle, #131b2c)',
                        backgroundColor: isRecording ? 'var(--bg-active, #253349)' : 'transparent',
                      }}
                    >
                      <div>
                        <div style={{ display: 'flex', alignItems: 'center', gap: '8px' }}>
                          <span
                            style={{
                              fontSize: '9px',
                              textTransform: 'uppercase',
                              padding: '2px 4px',
                              borderRadius: '2px',
                              backgroundColor: 'var(--bg-hover, #1e293b)',
                              color: 'var(--text-muted, #94a3b8)',
                              fontWeight: 600,
                            }}
                          >
                            {action.category}
                          </span>
                          <span style={{ fontSize: 'var(--font-sm, 12px)', fontWeight: 500 }}>
                            {action.label}
                          </span>
                        </div>
                        <div style={{ fontSize: '10px', color: 'var(--text-muted, #64748b)', marginTop: '2px' }}>
                          {action.description}
                        </div>
                      </div>

                      <div style={{ display: 'flex', alignItems: 'center', gap: '6px' }}>
                        {isRecording ? (
                          <span
                            style={{
                              fontSize: '10px',
                              padding: '4px 8px',
                              backgroundColor: 'var(--accent-blue, #0284c7)',
                              color: '#ffffff',
                              borderRadius: '3px',
                              animation: 'pulse 1s infinite',
                            }}
                          >
                            Press key combination (Esc to cancel)...
                          </span>
                        ) : (
                          <>
                            {currentCombo ? (
                              <span
                                style={{
                                  fontSize: '10px',
                                  fontFamily: 'monospace',
                                  padding: '3px 8px',
                                  backgroundColor: 'var(--bg-input, #131b2c)',
                                  border: '1px solid var(--border-button, #334155)',
                                  borderRadius: '3px',
                                  color: 'var(--accent-cyan, #38bdf8)',
                                }}
                              >
                                {currentCombo}
                              </span>
                            ) : (
                              <span style={{ fontSize: '10px', color: 'var(--text-muted, #64748b)', fontStyle: 'italic' }}>
                                None
                              </span>
                            )}

                            <button
                              onClick={() => setRecordingActionId(action.id)}
                              style={{
                                padding: '3px 8px',
                                fontSize: '10px',
                                backgroundColor: 'var(--bg-hover, #1e293b)',
                                color: 'var(--text-secondary, #cbd5e1)',
                                border: '1px solid var(--border-button, #334155)',
                                borderRadius: '3px',
                                cursor: 'pointer',
                              }}
                            >
                              Edit
                            </button>

                            {currentCombo && (
                              <button
                                onClick={() => handleClearKeybind(action.id)}
                                style={{
                                  padding: '3px 6px',
                                  fontSize: '10px',
                                  backgroundColor: 'transparent',
                                  color: 'var(--text-muted, #64748b)',
                                  border: 'none',
                                  cursor: 'pointer',
                                }}
                                title="Remove keybind"
                              >
                                Clear
                              </button>
                            )}
                          </>
                        )}
                      </div>
                    </div>
                  )
                })}
              </div>
            </div>
          )}

          {/* TAB 4: SIMULATION */}
          {activeTab === 'simulation' && (
            <div>
              <div style={{ fontSize: 'var(--font-title, 15px)', fontWeight: 600, marginBottom: '4px' }}>
                Simulation Engine Preferences
              </div>
              <div style={{ fontSize: 'var(--font-xs, 11px)', color: 'var(--text-muted, #64748b)', marginBottom: '20px' }}>
                Fine-tune non-linear Newton-Raphson convergence, damping, and thermal dissipation models.
              </div>

              <div style={{ display: 'flex', flexDirection: 'column', gap: '16px' }}>
                <div>
                  <label style={{ display: 'block', fontSize: 'var(--font-sm, 12px)', fontWeight: 600, marginBottom: '4px' }}>
                    Newton-Raphson Max Iterations: {settings.maxIterations}
                  </label>
                  <input
                    type="range"
                    min="20"
                    max="500"
                    step="10"
                    value={settings.maxIterations}
                    onChange={(e) => onUpdateSettings({ maxIterations: Number(e.target.value) })}
                    style={{ width: '320px', cursor: 'pointer' }}
                  />
                </div>

                <div>
                  <label style={{ display: 'block', fontSize: 'var(--font-sm, 12px)', fontWeight: 600, marginBottom: '4px' }}>
                    Non-Linear Relative Tolerance (Tol): {settings.tolerance.toExponential(1)}
                  </label>
                  <select
                    value={settings.tolerance.toString()}
                    onChange={(e) => onUpdateSettings({ tolerance: Number(e.target.value) })}
                    style={{
                      padding: '4px 8px',
                      backgroundColor: 'var(--bg-input, #131b2c)',
                      border: '1px solid var(--border-button, #334155)',
                      borderRadius: '3px',
                      color: 'var(--text-primary, #f8fafc)',
                      fontSize: 'var(--font-sm, 11px)',
                    }}
                  >
                    <option value="0.001">1e-3 (Fastest convergence)</option>
                    <option value="0.0001">1e-4 (Standard balanced default)</option>
                    <option value="0.00001">1e-5 (High precision)</option>
                    <option value="0.000001">1e-6 (Ultra-high precision)</option>
                  </select>
                </div>
              </div>
            </div>
          )}
        </div>
      </div>
    </Modal>
  )
}
