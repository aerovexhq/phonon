import React, { useState } from 'react'
import { Modal } from './Modal'

export interface SymbolPrimitive {
  type: 'line' | 'rect' | 'circle' | 'text'
  x1: number
  y1: number
  x2: number
  y2: number
  radius?: number
  text?: string
}

export interface SymbolPinDef {
  id: number
  name: string
  direction: 'input' | 'output' | 'bidirectional' | 'passive'
  x: number
  y: number
}

export interface CustomSymbolDefinition {
  id: string
  name: string
  prefix: string
  category: string
  primitives: SymbolPrimitive[]
  pins: SymbolPinDef[]
  designatorOffset: { x: number; y: number }
  valueOffset: { x: number; y: number }
  spiceSubcircuit?: string
}

interface SymbolEditorModalProps {
  isOpen: boolean
  onClose: () => void
  onSaveSymbol: (symbol: CustomSymbolDefinition) => void
}

export const SymbolEditorModal: React.FC<SymbolEditorModalProps> = ({
  isOpen,
  onClose,
  onSaveSymbol,
}) => {
  const [symbolId, setSymbolId] = useState<string>('custom_ic')
  const [symbolName, setSymbolName] = useState<string>('Custom Module')
  const [prefix, setPrefix] = useState<string>('U')
  const [category, setCategory] = useState<string>('Integrated Circuits')
  const [activeTool, setActiveTool] = useState<'select' | 'rect' | 'line' | 'circle' | 'pin' | 'text'>('select')

  const [primitives, setPrimitives] = useState<SymbolPrimitive[]>([
    { type: 'rect', x1: -30, y1: -20, x2: 30, y2: 20 },
  ])

  const [pins, setPins] = useState<SymbolPinDef[]>([
    { id: 1, name: 'IN', direction: 'input', x: -40, y: 0 },
    { id: 2, name: 'OUT', direction: 'output', x: 40, y: 0 },
  ])

  const [designatorOffset, setDesignatorOffset] = useState<{ x: number; y: number }>({ x: 35, y: -15 })
  const [valueOffset, setValueOffset] = useState<{ x: number; y: number }>({ x: 35, y: 5 })

  const [spiceTemplate, setSpiceTemplate] = useState<string>('.subckt CUSTOM_IC 1 2\n* Internal macro-model nodes\nR1 1 2 10k\n.ends')

  const [newPinName, setNewPinName] = useState<string>('P1')
  const [newPinDir, setNewPinDir] = useState<'input' | 'output' | 'bidirectional' | 'passive'>('passive')

  // Check collision between labels and body
  const bodyMinX = primitives.reduce((acc, p) => Math.min(acc, p.x1, p.x2), -30)
  const bodyMaxX = primitives.reduce((acc, p) => Math.max(acc, p.x1, p.x2), 30)
  const bodyMinY = primitives.reduce((acc, p) => Math.min(acc, p.y1, p.y2), -20)
  const bodyMaxY = primitives.reduce((acc, p) => Math.max(acc, p.y1, p.y2), 20)

  const hasCollision =
    (designatorOffset.x >= bodyMinX && designatorOffset.x <= bodyMaxX && designatorOffset.y >= bodyMinY && designatorOffset.y <= bodyMaxY) ||
    (valueOffset.x >= bodyMinX && valueOffset.x <= bodyMaxX && valueOffset.y >= bodyMinY && valueOffset.y <= bodyMaxY)

  const handleAutoAvoid = () => {
    setDesignatorOffset({ x: bodyMaxX + 10, y: bodyMinY + 5 })
    setValueOffset({ x: bodyMaxX + 10, y: bodyMinY + 22 })
  }

  const handleCanvasClick = (e: React.MouseEvent<SVGSVGElement>) => {
    const rect = e.currentTarget.getBoundingClientRect()
    const screenX = e.clientX - rect.left - rect.width / 2
    const screenY = e.clientY - rect.top - rect.height / 2
    const snappedX = Math.round(screenX / 10) * 10
    const snappedY = Math.round(screenY / 10) * 10

    if (activeTool === 'pin') {
      const newId = pins.length > 0 ? Math.max(...pins.map((p) => p.id)) + 1 : 1
      setPins((prev) => [
        ...prev,
        { id: newId, name: newPinName, direction: newPinDir, x: snappedX, y: snappedY },
      ])
    } else if (activeTool === 'circle') {
      setPrimitives((prev) => [
        ...prev,
        { type: 'circle', x1: snappedX, y1: snappedY, x2: snappedX, y2: snappedY, radius: 15 },
      ])
    }
  }

  const handleSave = () => {
    onSaveSymbol({
      id: symbolId,
      name: symbolName,
      prefix,
      category,
      primitives,
      pins,
      designatorOffset,
      valueOffset,
      spiceSubcircuit: spiceTemplate,
    })
    onClose()
  }

  return (
    <Modal
      isOpen={isOpen}
      onClose={onClose}
      title="Component Symbol & Shape Editor (Logisim/KiCad Style)"
      width="85%"
      height="85%"
    >
      <div style={{ display: 'flex', flex: 1, gap: '16px', minHeight: 0, height: '100%' }}>
        {/* Left Toolbar */}
        <div style={{ width: '180px', display: 'flex', flexDirection: 'column', gap: '12px', borderRight: '1px solid var(--border-color, #1a2233)', paddingRight: '12px' }}>
          <div>
            <div style={{ fontSize: '11px', fontWeight: 600, color: '#94a3b8', textTransform: 'uppercase', marginBottom: '6px' }}>Tools</div>
            <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: '4px' }}>
              {(['select', 'rect', 'line', 'circle', 'pin', 'text'] as const).map((tool) => (
                <button
                  key={tool}
                  onClick={() => setActiveTool(tool)}
                  style={{
                    padding: '4px 8px',
                    backgroundColor: activeTool === tool ? '#0284c7' : 'var(--bg-hover, #1e293b)',
                    border: '1px solid var(--border-button, #334155)',
                    borderRadius: '2px',
                    color: '#f8fafc',
                    fontSize: '11px',
                    cursor: 'pointer',
                    textTransform: 'capitalize',
                  }}
                >
                  {tool}
                </button>
              ))}
            </div>
          </div>

          <div>
            <div style={{ fontSize: '11px', fontWeight: 600, color: '#94a3b8', textTransform: 'uppercase', marginBottom: '6px' }}>Pin Placement</div>
            <input
              type="text"
              value={newPinName}
              onChange={(e) => setNewPinName(e.target.value)}
              placeholder="Pin Name"
              style={{
                width: '100%',
                backgroundColor: 'var(--bg-input, #131b2c)',
                border: '1px solid var(--border-button, #334155)',
                color: '#f8fafc',
                padding: '4px 6px',
                fontSize: '11px',
                borderRadius: '2px',
                marginBottom: '4px',
              }}
            />
            <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: '2px' }}>
              {(['input', 'output', 'bidirectional', 'passive'] as const).map((dir) => (
                <button
                  key={dir}
                  onClick={() => setNewPinDir(dir)}
                  style={{
                    padding: '2px 4px',
                    backgroundColor: newPinDir === dir ? '#0369a1' : 'transparent',
                    border: '1px solid var(--border-subtle, #1a2233)',
                    color: '#cbd5e1',
                    fontSize: '9px',
                    cursor: 'pointer',
                    borderRadius: '2px',
                    textTransform: 'capitalize',
                  }}
                >
                  {dir}
                </button>
              ))}
            </div>
          </div>

          <button
            onClick={handleAutoAvoid}
            style={{
              padding: '6px 10px',
              backgroundColor: 'var(--bg-hover, #1e293b)',
              border: '1px solid #0284c7',
              borderRadius: '3px',
              color: '#38bdf8',
              fontSize: '11px',
              fontWeight: 500,
              cursor: 'pointer',
              marginTop: 'auto',
            }}
          >
            Auto-Avoid Collisions
          </button>
        </div>

        {/* Center Vector Canvas */}
        <div style={{ flex: 1, position: 'relative', backgroundColor: '#070a10', border: '1px solid var(--border-color, #1a2233)', borderRadius: '4px', overflow: 'hidden' }}>
          <svg
            style={{ width: '100%', height: '100%', cursor: activeTool === 'pin' ? 'crosshair' : 'default' }}
            onClick={handleCanvasClick}
          >
            <defs>
              <pattern id="editor-grid" width="20" height="20" patternUnits="userSpaceOnUse">
                <circle cx="10" cy="10" r="0.8" fill="#1e293b" />
              </pattern>
            </defs>
            <rect width="100%" height="100%" fill="url(#editor-grid)" />

            <g transform="translate(250, 180)">
              {/* Origin axes */}
              <line x1="-240" y1="0" x2="240" y2="0" stroke="#131b2c" strokeWidth="1" />
              <line x1="0" y1="-170" x2="0" y2="170" stroke="#131b2c" strokeWidth="1" />

              {/* Render primitives */}
              {primitives.map((prim, idx) => {
                if (prim.type === 'rect') {
                  const x = Math.min(prim.x1, prim.x2)
                  const y = Math.min(prim.y1, prim.y2)
                  const w = Math.abs(prim.x2 - prim.x1)
                  const h = Math.abs(prim.y2 - prim.y1)
                  return <rect key={idx} x={x} y={y} width={w} height={h} fill="rgba(56, 189, 248, 0.05)" stroke="#38bdf8" strokeWidth="1.5" />
                } else if (prim.type === 'circle') {
                  return <circle key={idx} cx={prim.x1} cy={prim.y1} r={prim.radius || 15} fill="rgba(56, 189, 248, 0.05)" stroke="#38bdf8" strokeWidth="1.5" />
                } else if (prim.type === 'line') {
                  return <line key={idx} x1={prim.x1} y1={prim.y1} x2={prim.x2} y2={prim.y2} stroke="#38bdf8" strokeWidth="1.5" />
                }
                return null
              })}

              {/* Render terminal pins */}
              {pins.map((pin) => (
                <g key={pin.id}>
                  <line x1={pin.x} y1={pin.y} x2={pin.x > 0 ? pin.x - 10 : pin.x + 10} y2={pin.y} stroke="#22c55e" strokeWidth="1.5" />
                  <circle cx={pin.x} cy={pin.y} r="3" fill="#22c55e" />
                  <text x={pin.x > 0 ? pin.x - 14 : pin.x + 14} y={pin.y + 3} fill="#94a3b8" fontSize="9" textAnchor={pin.x > 0 ? 'end' : 'start'} fontFamily="monospace">{pin.name}</text>
                </g>
              ))}

              {/* Labels with collision indicator */}
              <text x={designatorOffset.x} y={designatorOffset.y} fill={hasCollision ? '#ef4444' : '#38bdf8'} fontSize="11" fontFamily="monospace" fontWeight="600">{prefix}1</text>
              <text x={valueOffset.x} y={valueOffset.y} fill={hasCollision ? '#ef4444' : '#94a3b8'} fontSize="10" fontFamily="monospace">{symbolName}</text>
            </g>
          </svg>

          {hasCollision && (
            <div style={{ position: 'absolute', bottom: '10px', left: '10px', padding: '4px 8px', backgroundColor: 'rgba(239, 68, 68, 0.2)', border: '1px solid #ef4444', borderRadius: '3px', color: '#fca5a5', fontSize: '11px' }}>
              Warning: Labels collide with component body. Click 'Auto-Avoid Collisions' to resolve.
            </div>
          )}
        </div>

        {/* Right Metadata & Subcircuit Config */}
        <div style={{ width: '240px', display: 'flex', flexDirection: 'column', gap: '10px', borderLeft: '1px solid var(--border-color, #1a2233)', paddingLeft: '12px' }}>
          <div style={{ fontSize: '11px', fontWeight: 600, color: '#94a3b8', textTransform: 'uppercase' }}>Module Properties</div>
          
          <div>
            <label style={{ display: 'block', fontSize: '10px', color: '#94a3b8', marginBottom: '2px' }}>Identifier</label>
            <input
              type="text"
              value={symbolId}
              onChange={(e) => setSymbolId(e.target.value)}
              style={{ width: '100%', backgroundColor: 'var(--bg-input, #131b2c)', border: '1px solid var(--border-button, #334155)', color: '#f8fafc', padding: '3px 6px', fontSize: '11px', borderRadius: '2px' }}
            />
          </div>

          <div>
            <label style={{ display: 'block', fontSize: '10px', color: '#94a3b8', marginBottom: '2px' }}>Display Name</label>
            <input
              type="text"
              value={symbolName}
              onChange={(e) => setSymbolName(e.target.value)}
              style={{ width: '100%', backgroundColor: 'var(--bg-input, #131b2c)', border: '1px solid var(--border-button, #334155)', color: '#f8fafc', padding: '3px 6px', fontSize: '11px', borderRadius: '2px' }}
            />
          </div>

          <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: '8px' }}>
            <div>
              <label style={{ display: 'block', fontSize: '10px', color: '#94a3b8', marginBottom: '2px' }}>Prefix</label>
              <input
                type="text"
                value={prefix}
                onChange={(e) => setPrefix(e.target.value)}
                style={{ width: '100%', backgroundColor: 'var(--bg-input, #131b2c)', border: '1px solid var(--border-button, #334155)', color: '#f8fafc', padding: '3px 6px', fontSize: '11px', borderRadius: '2px' }}
              />
            </div>
            <div>
              <label style={{ display: 'block', fontSize: '10px', color: '#94a3b8', marginBottom: '2px' }}>Category</label>
              <input
                type="text"
                value={category}
                onChange={(e) => setCategory(e.target.value)}
                style={{ width: '100%', backgroundColor: 'var(--bg-input, #131b2c)', border: '1px solid var(--border-button, #334155)', color: '#f8fafc', padding: '3px 6px', fontSize: '11px', borderRadius: '2px' }}
              />
            </div>
          </div>

          <div>
            <label style={{ display: 'block', fontSize: '10px', color: '#94a3b8', marginBottom: '2px' }}>SPICE Subcircuit Template</label>
            <textarea
              value={spiceTemplate}
              onChange={(e) => setSpiceTemplate(e.target.value)}
              rows={4}
              style={{ width: '100%', backgroundColor: 'var(--bg-input, #131b2c)', border: '1px solid var(--border-button, #334155)', color: '#f8fafc', padding: '4px', fontSize: '10px', fontFamily: 'monospace', borderRadius: '2px', resize: 'vertical' }}
            />
          </div>

          <div style={{ marginTop: 'auto', display: 'flex', gap: '8px' }}>
            <button
              onClick={handleSave}
              style={{
                flex: 1,
                padding: '6px 12px',
                backgroundColor: '#0284c7',
                border: 'none',
                borderRadius: '3px',
                color: '#ffffff',
                fontSize: '11px',
                fontWeight: 600,
                cursor: 'pointer',
              }}
            >
              Save Symbol
            </button>
            <button
              onClick={onClose}
              style={{
                padding: '6px 12px',
                backgroundColor: 'transparent',
                border: '1px solid var(--border-button, #334155)',
                borderRadius: '3px',
                color: '#94a3b8',
                fontSize: '11px',
                cursor: 'pointer',
              }}
            >
              Cancel
            </button>
          </div>
        </div>
      </div>
    </Modal>
  )
}
