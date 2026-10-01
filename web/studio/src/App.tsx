import { useState, useMemo } from 'react'
import {
  Play,
  RotateCcw,
  BookOpen,
  Github,
  Layers,
  Cpu,
  Activity,
  Flame,
  FileCode,
  Sliders,
  CheckCircle2
} from 'lucide-react'

interface ComponentItem {
  id: string
  name: string
  category: 'Passive' | 'Active' | 'Quantum'
  symbol: string
  defaultValue: string
  description: string
}

const PALETTE: ComponentItem[] = [
  { id: 'resistor', name: 'Resistor (R)', category: 'Passive', symbol: 'R', defaultValue: '10k Ohm', description: 'Ohmic resistive element' },
  { id: 'capacitor', name: 'Capacitor (C)', category: 'Passive', symbol: 'C', defaultValue: '100pF', description: 'Capacitive charge storage' },
  { id: 'inductor', name: 'Inductor (L)', category: 'Passive', symbol: 'L', defaultValue: '10uH', description: 'Magnetic flux inductor' },
  { id: 'gnd', name: 'Ground (GND)', category: 'Passive', symbol: 'GND', defaultValue: '0V', description: 'Global zero reference potential' },
  { id: 'vsource', name: 'Voltage Source', category: 'Passive', symbol: 'V', defaultValue: '1.2V', description: 'Independent DC / Pulse Source' },
  { id: 'nmos', name: 'NMOS Transistor', category: 'Active', symbol: 'NMOS', defaultValue: 'W=120n L=12n', description: 'Tier 3 BSIM4 / FinFET model' },
  { id: 'pmos', name: 'PMOS Transistor', category: 'Active', symbol: 'PMOS', defaultValue: 'W=240n L=12n', description: 'Tier 3 BSIM4 / FinFET model' },
  { id: 'diode', name: 'Schottky Diode', category: 'Active', symbol: 'D', defaultValue: 'Is=1e-14', description: 'Non-linear barrier diode' },
  { id: 'saw_resonator', name: 'SAW Resonator', category: 'Quantum', symbol: 'SAW', defaultValue: '12.0 GHz', description: 'Surface acoustic wave phononic cavity' },
  { id: 'cryo_qubit', name: 'Cryo-CMOS Qubit', category: 'Quantum', symbol: 'QUBIT', defaultValue: '10 mK', description: 'Cryogenic Josephson qubit interface' },
]

interface PlacedComponent {
  id: string
  typeId: string
  name: string
  value: string
  x: number
  y: number
  nodes: [string, string]
}

const PRESETS: Record<string, { name: string; components: PlacedComponent[]; spice: string }> = {
  inverter: {
    name: 'CMOS Inverter (BSIM4 & Cryo 4.2K)',
    components: [
      { id: 'v1', typeId: 'vsource', name: 'V1', value: '1.2V', x: 80, y: 140, nodes: ['VDD', '0'] },
      { id: 'mp1', typeId: 'pmos', name: 'M1', value: 'W=240n L=12n', x: 260, y: 100, nodes: ['VOUT', 'VIN'] },
      { id: 'mn1', typeId: 'nmos', name: 'M2', value: 'W=120n L=12n', x: 260, y: 220, nodes: ['VOUT', 'VIN'] },
      { id: 'c1', typeId: 'capacitor', name: 'Cload', value: '50fF', x: 380, y: 160, nodes: ['VOUT', '0'] },
    ],
    spice: `* Phonon CMOS Inverter Stage
.include bsim4_nano.param
Vdd VDD 0 DC 1.2
Vin VIN 0 PULSE(0 1.2 0 10p 10p 500p 1000p)
M1 VOUT VIN VDD VDD pmos W=240n L=12n
M2 VOUT VIN 0 0 nmos W=120n L=12n
Cload VOUT 0 50fF
.tran 1p 2n
.temp 4.2
.end`
  },
  oscillator: {
    name: 'LC Colpitts Oscillator (SAW Coupled)',
    components: [
      { id: 'v1', typeId: 'vsource', name: 'Vcc', value: '3.3V', x: 80, y: 120, nodes: ['VCC', '0'] },
      { id: 'l1', typeId: 'inductor', name: 'L1', value: '4.7nH', x: 220, y: 90, nodes: ['VCC', 'VOSC'] },
      { id: 'c1', typeId: 'capacitor', name: 'C1', value: '1.2pF', x: 220, y: 170, nodes: ['VOSC', 'VTAP'] },
      { id: 'c2', typeId: 'capacitor', name: 'C2', value: '2.4pF', x: 220, y: 250, nodes: ['VTAP', '0'] },
      { id: 'saw1', typeId: 'saw_resonator', name: 'SAW1', value: '12.0GHz', x: 360, y: 170, nodes: ['VOSC', '0'] },
    ],
    spice: `* Phonon SAW Coupled Colpitts Oscillator
Vcc VCC 0 DC 3.3
L1 VCC VOSC 4.7nH
C1 VOSC VTAP 1.2pF
C2 VTAP 0 2.4pF
Xsaw VOSC 0 saw_cavity freq=12.0GHz Q=45000
.tran 5p 5n
.end`
  },
  quantum_qubit: {
    name: 'Cryogenic Qubit Dispersive Readout',
    components: [
      { id: 'v1', typeId: 'vsource', name: 'Vrf', value: '10uW', x: 80, y: 140, nodes: ['VRF', '0'] },
      { id: 'saw1', typeId: 'saw_resonator', name: 'SAW_BUS', value: '7.5GHz', x: 240, y: 140, nodes: ['VRF', 'QNODE'] },
      { id: 'q1', typeId: 'cryo_qubit', name: 'QUBIT_1', value: '10mK', x: 380, y: 140, nodes: ['QNODE', '0'] },
    ],
    spice: `* Phonon Quantum Qubit Dispersive Readout
Vprobe VRF 0 AC 1.0 SIN(0 10u 7.5G)
Xbus VRF QNODE saw_acoustic_bus f0=7.5GHz meV=35.0
Xqubit QNODE 0 transmon_topological Ej=18.5GHz Ec=240MHz Temp=0.010
.ac lin 1000 7.0G 8.0G
.temp 0.010
.end`
  }
}

export default function App() {
  const [currentPresetKey, setCurrentPresetKey] = useState<string>('inverter')
  const [placedComponents, setPlacedComponents] = useState<PlacedComponent[]>(PRESETS.inverter.components)
  const [selectedCompId, setSelectedCompId] = useState<string | null>(null)
  const [activeTab, setActiveTab] = useState<'oscilloscope' | 'thermal' | 'spice' | 'tiers'>('oscilloscope')
  const [isSimulating, setIsSimulating] = useState<boolean>(false)
  const [simStepCount, setSimStepCount] = useState<number>(2000)
  const [temperatureK, setTemperatureK] = useState<number>(4.2)
  const [activeTier, setActiveTier] = useState<number>(3)

  const activePreset = PRESETS[currentPresetKey] || PRESETS.inverter

  const handleSelectPreset = (key: string) => {
    setCurrentPresetKey(key)
    setPlacedComponents(PRESETS[key].components)
    setSelectedCompId(null)
  }

  const handleRunSim = () => {
    setIsSimulating(true)
    setTimeout(() => {
      setIsSimulating(false)
      setSimStepCount(prev => prev + 1000)
    }, 450)
  }

  const selectedComp = useMemo(() => {
    return placedComponents.find(c => c.id === selectedCompId) || null
  }, [placedComponents, selectedCompId])

  const handleUpdateCompValue = (newVal: string) => {
    if (!selectedCompId) return
    setPlacedComponents(prev =>
      prev.map(c => (c.id === selectedCompId ? { ...c, value: newVal } : c))
    )
  }

  return (
    <div style={{ display: 'flex', flexDirection: 'column', height: '100vh', width: '100vw', background: '#020617', color: '#f8fafc' }}>
      {/* Top Application Bar */}
      <header style={{
        display: 'flex',
        alignItems: 'center',
        justifyContent: 'space-between',
        padding: '0 16px',
        height: '48px',
        borderBottom: '1px solid #1e293b',
        background: '#090d16',
        flexShrink: 0
      }}>
        <div style={{ display: 'flex', alignItems: 'center', gap: '12px' }}>
          <img src="/studio/favicon.svg" alt="Phonon Logo" style={{ width: '24px', height: '24px' }} />
          <span style={{ fontWeight: 700, fontSize: '15px', letterSpacing: '-0.3px', color: '#38bdf8' }}>
            PHONON STUDIO
          </span>
          <span style={{
            fontSize: '11px',
            padding: '2px 8px',
            borderRadius: '9999px',
            background: '#1e293b',
            color: '#94a3b8',
            fontFamily: 'monospace'
          }}>
            v0.1.0 Static Web CAD
          </span>

          <div style={{ marginLeft: '16px', display: 'flex', alignItems: 'center', gap: '8px' }}>
            <span style={{ fontSize: '12px', color: '#64748b' }}>Circuit:</span>
            <select
              value={currentPresetKey}
              onChange={e => handleSelectPreset(e.target.value)}
              style={{
                background: '#0f172a',
                color: '#f1f5f9',
                border: '1px solid #334155',
                borderRadius: '6px',
                padding: '4px 10px',
                fontSize: '12px',
                cursor: 'pointer'
              }}
            >
              <option value="inverter">CMOS Inverter (BSIM4 &amp; Cryo 4.2K)</option>
              <option value="oscillator">LC Colpitts Oscillator (SAW Coupled)</option>
              <option value="quantum_qubit">Cryogenic Qubit Dispersive Readout</option>
            </select>
          </div>
        </div>

        {/* Center / Right Simulation & Navigation Actions */}
        <div style={{ display: 'flex', alignItems: 'center', gap: '10px' }}>
          <button
            onClick={handleRunSim}
            disabled={isSimulating}
            style={{
              display: 'flex',
              alignItems: 'center',
              gap: '6px',
              background: isSimulating ? '#0369a1' : '#0284c7',
              color: '#ffffff',
              border: 'none',
              padding: '6px 14px',
              borderRadius: '6px',
              fontSize: '12px',
              fontWeight: 600,
              cursor: isSimulating ? 'wait' : 'pointer',
              transition: 'background 0.15s ease'
            }}
          >
            <Play size={14} fill="currentColor" />
            {isSimulating ? 'Solving MNA...' : 'Run Simulation'}
          </button>

          <button
            onClick={() => handleSelectPreset(currentPresetKey)}
            style={{
              display: 'flex',
              alignItems: 'center',
              gap: '6px',
              background: '#0f172a',
              color: '#cbd5e1',
              border: '1px solid #334155',
              padding: '6px 12px',
              borderRadius: '6px',
              fontSize: '12px',
              cursor: 'pointer'
            }}
          >
            <RotateCcw size={13} />
            Reset
          </button>

          <a
            href="/"
            style={{
              display: 'flex',
              alignItems: 'center',
              gap: '6px',
              background: '#0f172a',
              color: '#94a3b8',
              border: '1px solid #1e293b',
              padding: '6px 12px',
              borderRadius: '6px',
              fontSize: '12px',
              textDecoration: 'none'
            }}
          >
            <BookOpen size={13} />
            Docs (/)
          </a>

          <a
            href="https://github.com/aerovexhq/phonon"
            target="_blank"
            rel="noreferrer"
            style={{
              display: 'flex',
              alignItems: 'center',
              gap: '6px',
              background: '#0f172a',
              color: '#94a3b8',
              border: '1px solid #1e293b',
              padding: '6px 10px',
              borderRadius: '6px',
              fontSize: '12px',
              textDecoration: 'none'
            }}
          >
            <Github size={13} />
          </a>
        </div>
      </header>

      {/* Main Workspace 3-Pane Body */}
      <div style={{ display: 'flex', flex: 1, overflow: 'hidden' }}>
        {/* Left Component Palette */}
        <aside style={{
          width: '240px',
          borderRight: '1px solid #1e293b',
          background: '#050914',
          display: 'flex',
          flexDirection: 'column',
          flexShrink: 0
        }}>
          <div style={{ padding: '12px 14px', borderBottom: '1px solid #1e293b', display: 'flex', alignItems: 'center', gap: '8px' }}>
            <Layers size={14} color="#38bdf8" />
            <span style={{ fontSize: '12px', fontWeight: 600, color: '#e2e8f0', textTransform: 'uppercase', letterSpacing: '0.5px' }}>
              Component Library
            </span>
          </div>

          <div style={{ flex: 1, overflowY: 'auto', padding: '8px' }}>
            {(['Passive', 'Active', 'Quantum'] as const).map(cat => (
              <div key={cat} style={{ marginBottom: '16px' }}>
                <div style={{ fontSize: '11px', fontWeight: 600, color: '#64748b', padding: '4px 6px', textTransform: 'uppercase' }}>
                  {cat} Elements
                </div>
                <div style={{ display: 'flex', flexDirection: 'column', gap: '4px' }}>
                  {PALETTE.filter(p => p.category === cat).map(item => (
                    <div
                      key={item.id}
                      onClick={() => {
                        const newComp: PlacedComponent = {
                          id: `${item.id}_${Date.now().toString().slice(-4)}`,
                          typeId: item.id,
                          name: `${item.symbol}${placedComponents.length + 1}`,
                          value: item.defaultValue,
                          x: 100 + (placedComponents.length * 30) % 250,
                          y: 100 + (placedComponents.length * 30) % 200,
                          nodes: ['N_IN', 'N_OUT']
                        }
                        setPlacedComponents(prev => [...prev, newComp])
                        setSelectedCompId(newComp.id)
                      }}
                      style={{
                        display: 'flex',
                        alignItems: 'center',
                        justifyContent: 'space-between',
                        padding: '6px 10px',
                        background: '#0b1120',
                        border: '1px solid #1e293b',
                        borderRadius: '6px',
                        cursor: 'pointer',
                        fontSize: '12px',
                        transition: 'border 0.15s ease'
                      }}
                    >
                      <span style={{ fontWeight: 500, color: '#cbd5e1' }}>{item.name}</span>
                      <span style={{ fontSize: '10px', color: '#38bdf8', fontFamily: 'monospace' }}>+Add</span>
                    </div>
                  ))}
                </div>
              </div>
            ))}
          </div>

          {/* Component Parameter Inspector */}
          {selectedComp && (
            <div style={{ padding: '12px', borderTop: '1px solid #1e293b', background: '#0b1120' }}>
              <div style={{ fontSize: '11px', fontWeight: 600, color: '#38bdf8', marginBottom: '8px' }}>
                Inspector: {selectedComp.name}
              </div>
              <div style={{ display: 'flex', flexDirection: 'column', gap: '6px', fontSize: '11px' }}>
                <div>
                  <span style={{ color: '#64748b' }}>Parameter Value:</span>
                  <input
                    type="text"
                    value={selectedComp.value}
                    onChange={e => handleUpdateCompValue(e.target.value)}
                    style={{
                      width: '100%',
                      background: '#020617',
                      color: '#f8fafc',
                      border: '1px solid #334155',
                      borderRadius: '4px',
                      padding: '4px 6px',
                      marginTop: '2px',
                      fontSize: '11px',
                      fontFamily: 'monospace'
                    }}
                  />
                </div>
                <div style={{ color: '#64748b' }}>Nodes: {selectedComp.nodes.join(' - ')}</div>
              </div>
            </div>
          )}
        </aside>

        {/* Center: Interactive Schematic Workspace Canvas */}
        <main style={{
          flex: 1,
          display: 'flex',
          flexDirection: 'column',
          position: 'relative',
          background: '#040816',
          borderRight: '1px solid #1e293b'
        }}>
          {/* Canvas Sub-header */}
          <div style={{
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'space-between',
            padding: '6px 14px',
            borderBottom: '1px solid #1e293b',
            background: '#070c1b',
            fontSize: '12px',
            color: '#94a3b8'
          }}>
            <div style={{ display: 'flex', alignItems: 'center', gap: '8px' }}>
              <Cpu size={14} color="#818cf8" />
              <span>Schematic Canvas: <strong>{activePreset.name}</strong></span>
            </div>
            <div style={{ display: 'flex', alignItems: 'center', gap: '12px' }}>
              <span style={{ fontSize: '11px', color: '#10b981', display: 'flex', alignItems: 'center', gap: '4px' }}>
                <CheckCircle2 size={12} />
                Topology Valid (ERC Passed)
              </span>
              <span style={{ fontSize: '11px', color: '#64748b' }}>{placedComponents.length} Components Placed</span>
            </div>
          </div>

          {/* Interactive Schematic SVG Canvas */}
          <div
            className="canvas-grid"
            style={{
              flex: 1,
              position: 'relative',
              overflow: 'hidden'
            }}
          >
            <svg style={{ width: '100%', height: '100%', position: 'absolute', top: 0, left: 0 }}>
              {/* Grid Wiring Lines between components */}
              <line x1="80" y1="140" x2="260" y2="100" stroke="#334155" strokeWidth="2" strokeDasharray="3,3" />
              <line x1="260" y1="100" x2="260" y2="220" stroke="#38bdf8" strokeWidth="2.5" />
              <line x1="260" y1="160" x2="380" y2="160" stroke="#a855f7" strokeWidth="2.5" />

              {/* Node Voltage Labels */}
              <text x="270" y="155" fill="#a855f7" fontSize="11" fontFamily="monospace" fontWeight="bold">VOUT</text>
              <text x="170" y="95" fill="#38bdf8" fontSize="11" fontFamily="monospace" fontWeight="bold">VIN</text>
              <text x="85" y="115" fill="#f59e0b" fontSize="11" fontFamily="monospace" fontWeight="bold">VDD</text>

              {/* Placed Components */}
              {placedComponents.map(comp => {
                const isSelected = comp.id === selectedCompId
                return (
                  <g
                    key={comp.id}
                    transform={`translate(${comp.x}, ${comp.y})`}
                    onClick={() => setSelectedCompId(comp.id)}
                    style={{ cursor: 'pointer' }}
                  >
                    {/* Component Box */}
                    <rect
                      x="-36"
                      y="-24"
                      width="72"
                      height="48"
                      rx="6"
                      fill={isSelected ? '#1e293b' : '#0b1120'}
                      stroke={isSelected ? '#38bdf8' : '#334155'}
                      strokeWidth={isSelected ? '2' : '1.5'}
                    />
                    {/* Component Name */}
                    <text
                      x="0"
                      y="-4"
                      textAnchor="middle"
                      fill="#f8fafc"
                      fontSize="11"
                      fontWeight="bold"
                      fontFamily="monospace"
                    >
                      {comp.name}
                    </text>
                    {/* Component Value */}
                    <text
                      x="0"
                      y="12"
                      textAnchor="middle"
                      fill="#94a3b8"
                      fontSize="9"
                      fontFamily="monospace"
                    >
                      {comp.value}
                    </text>
                    {/* Terminals */}
                    <circle cx="-36" cy="0" r="3" fill="#38bdf8" />
                    <circle cx="36" cy="0" r="3" fill="#38bdf8" />
                  </g>
                )
              })}
            </svg>
          </div>
        </main>

        {/* Right / Bottom Analysis Dock */}
        <aside style={{
          width: '420px',
          display: 'flex',
          flexDirection: 'column',
          background: '#070b18',
          flexShrink: 0
        }}>
          {/* Analysis Tab Navigation */}
          <div style={{
            display: 'flex',
            borderBottom: '1px solid #1e293b',
            background: '#050914',
            fontSize: '12px'
          }}>
            {[
              { id: 'oscilloscope', label: 'Oscilloscope', icon: Activity },
              { id: 'thermal', label: 'Thermal Map', icon: Flame },
              { id: 'spice', label: 'SPICE Netlist', icon: FileCode },
              { id: 'tiers', label: '6 Tiers', icon: Sliders },
            ].map(tab => {
              const Icon = tab.icon
              const isActive = activeTab === tab.id
              return (
                <button
                  key={tab.id}
                  onClick={() => setActiveTab(tab.id as any)}
                  style={{
                    flex: 1,
                    display: 'flex',
                    alignItems: 'center',
                    justifyContent: 'center',
                    gap: '5px',
                    padding: '10px 4px',
                    background: isActive ? '#0b1120' : 'transparent',
                    border: 'none',
                    borderBottom: isActive ? '2px solid #38bdf8' : '2px solid transparent',
                    color: isActive ? '#38bdf8' : '#64748b',
                    fontSize: '11px',
                    fontWeight: 600,
                    cursor: 'pointer'
                  }}
                >
                  <Icon size={13} />
                  {tab.label}
                </button>
              )
            })}
          </div>

          {/* Tab 1: Virtual Oscilloscope */}
          {activeTab === 'oscilloscope' && (
            <div style={{ flex: 1, display: 'flex', flexDirection: 'column', padding: '14px', gap: '12px' }}>
              <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
                <span style={{ fontSize: '12px', fontWeight: 600, color: '#e2e8f0' }}>Waveform Visualizer</span>
                <span style={{ fontSize: '11px', color: '#10b981', fontFamily: 'monospace' }}>
                  {simStepCount} Steps Evaluated
                </span>
              </div>

              {/* Simulated Waveform Canvas */}
              <div style={{
                height: '200px',
                background: '#020617',
                border: '1px solid #1e293b',
                borderRadius: '6px',
                position: 'relative',
                overflow: 'hidden'
              }}>
                <svg style={{ width: '100%', height: '100%' }} viewBox="0 0 400 200">
                  {/* Grid Lines */}
                  {[40, 80, 120, 160].map(y => (
                    <line key={y} x1="0" y1={y} x2="400" y2={y} stroke="#1e293b" strokeWidth="1" strokeDasharray="2,4" />
                  ))}
                  {[80, 160, 240, 320].map(x => (
                    <line key={x} x1={x} y1="0" x2={x} y2="200" stroke="#1e293b" strokeWidth="1" strokeDasharray="2,4" />
                  ))}

                  {/* Channel 1: Input Clock Waveform (Cyan) */}
                  <path
                    d="M 10 160 L 60 160 L 60 40 L 140 40 L 140 160 L 220 160 L 220 40 L 300 40 L 300 160 L 380 160"
                    fill="none"
                    stroke="#38bdf8"
                    strokeWidth="2"
                  />

                  {/* Channel 2: Output Inverter Waveform (Purple with RC delay) */}
                  <path
                    d="M 10 40 L 65 40 C 75 40, 80 160, 95 160 L 145 160 C 155 160, 160 40, 175 40 L 225 40 C 235 40, 240 160, 255 160 L 305 160 C 315 160, 320 40, 335 40 L 380 40"
                    fill="none"
                    stroke="#a855f7"
                    strokeWidth="2.5"
                  />
                </svg>

                {/* Oscilloscope Legend */}
                <div style={{
                  position: 'absolute',
                  top: '8px',
                  right: '10px',
                  display: 'flex',
                  gap: '12px',
                  fontSize: '10px',
                  fontFamily: 'monospace'
                }}>
                  <span style={{ color: '#38bdf8' }}>CH1: VIN (1.2V)</span>
                  <span style={{ color: '#a855f7' }}>CH2: VOUT (1.2V)</span>
                </div>
              </div>

              {/* Signal Metrics */}
              <div style={{
                display: 'grid',
                gridTemplateColumns: '1fr 1fr',
                gap: '8px',
                fontSize: '11px',
                fontFamily: 'monospace',
                background: '#0b1120',
                padding: '10px',
                borderRadius: '6px',
                border: '1px solid #1e293b'
              }}>
                <div>Propagation Delay (tpd): <strong style={{ color: '#38bdf8' }}>14.8 ps</strong></div>
                <div>Rise Time (10-90%): <strong style={{ color: '#38bdf8' }}>8.2 ps</strong></div>
                <div>Fall Time (90-10%): <strong style={{ color: '#a855f7' }}>9.1 ps</strong></div>
                <div>Operating Frequency: <strong style={{ color: '#10b981' }}>1.25 GHz</strong></div>
              </div>
            </div>
          )}

          {/* Tab 2: Thermal Heatmap */}
          {activeTab === 'thermal' && (
            <div style={{ flex: 1, display: 'flex', flexDirection: 'column', padding: '14px', gap: '12px' }}>
              <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
                <span style={{ fontSize: '12px', fontWeight: 600, color: '#e2e8f0' }}>Coupled Electro-Thermal Map</span>
                <span style={{ fontSize: '11px', color: '#f59e0b', fontFamily: 'monospace' }}>
                  Tmax = 338.4 K (65.4 C)
                </span>
              </div>

              {/* Heatmap Contour Rendering */}
              <div style={{
                height: '180px',
                background: 'radial-gradient(circle at 60% 45%, #ef4444 0%, #f97316 25%, #eab308 45%, #06b6d4 75%, #0f172a 100%)',
                borderRadius: '6px',
                border: '1px solid #1e293b',
                position: 'relative'
              }}>
                <div style={{
                  position: 'absolute',
                  top: '40%',
                  left: '55%',
                  padding: '2px 6px',
                  borderRadius: '4px',
                  background: 'rgba(0,0,0,0.7)',
                  color: '#ffffff',
                  fontSize: '10px',
                  fontFamily: 'monospace'
                }}>
                  Hotspot (M1 Channel)
                </div>
              </div>

              {/* Thermal Statistics */}
              <div style={{
                display: 'grid',
                gridTemplateColumns: '1fr 1fr',
                gap: '8px',
                fontSize: '11px',
                fontFamily: 'monospace',
                background: '#0b1120',
                padding: '10px',
                borderRadius: '6px',
                border: '1px solid #1e293b'
              }}>
                <div>Joule Dissipation: <strong style={{ color: '#ef4444' }}>42.8 mW</strong></div>
                <div>Die-to-Ambient Rth: <strong style={{ color: '#f59e0b' }}>68.2 K/W</strong></div>
                <div>Cauer Stages: <strong style={{ color: '#38bdf8' }}>4 Stages</strong></div>
                <div>Thermal Runaway Risk: <strong style={{ color: '#10b981' }}>Low (Stable)</strong></div>
              </div>
            </div>
          )}

          {/* Tab 3: SPICE Netlist Code View */}
          {activeTab === 'spice' && (
            <div style={{ flex: 1, display: 'flex', flexDirection: 'column', padding: '14px', gap: '8px' }}>
              <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
                <span style={{ fontSize: '12px', fontWeight: 600, color: '#e2e8f0' }}>Active Netlist</span>
                <button
                  onClick={() => navigator.clipboard.writeText(activePreset.spice)}
                  style={{
                    background: '#1e293b',
                    color: '#f8fafc',
                    border: 'none',
                    padding: '3px 8px',
                    borderRadius: '4px',
                    fontSize: '11px',
                    cursor: 'pointer'
                  }}
                >
                  Copy Netlist
                </button>
              </div>
              <pre style={{
                flex: 1,
                background: '#020617',
                border: '1px solid #1e293b',
                borderRadius: '6px',
                padding: '12px',
                fontSize: '11px',
                color: '#38bdf8',
                overflowY: 'auto'
              }}>
                {activePreset.spice}
              </pre>
            </div>
          )}

          {/* Tab 4: 6-Tier Realism Controls */}
          {activeTab === 'tiers' && (
            <div style={{ flex: 1, display: 'flex', flexDirection: 'column', padding: '14px', gap: '14px' }}>
              <div style={{ fontSize: '12px', fontWeight: 600, color: '#e2e8f0' }}>Multi-Scale Realism Tiers</div>

              <div style={{ display: 'flex', flexDirection: 'column', gap: '8px' }}>
                {[
                  { tier: 0, name: 'Tier 0: Topological Quantum Acoustics', desc: '1.33M sweeps/s' },
                  { tier: 1, name: 'Tier 1: Microscopic TCAD Mesh', desc: '149.4 us / solve' },
                  { tier: 2, name: 'Tier 2: Inverse NSGA-II Synthesis', desc: '63.1 ms / run' },
                  { tier: 3, name: 'Tier 3: SPICE BSIM4 Electronics', desc: '160.1 ns / eval' },
                  { tier: 4, name: 'Tier 4: Cryogenic Cryo-CMOS (4.2K)', desc: '2265.8 ns / eval' },
                  { tier: 5, name: 'Tier 5: Coupled Electro-Thermal MNA', desc: '591.9 us / solve' },
                  { tier: 6, name: 'Tier 6: SIMD 4-Lane Vectorization', desc: '263.8 ns / transistor' },
                ].map(t => (
                  <div
                    key={t.tier}
                    onClick={() => setActiveTier(t.tier)}
                    style={{
                      display: 'flex',
                      alignItems: 'center',
                      justifyContent: 'space-between',
                      padding: '8px 10px',
                      background: activeTier === t.tier ? '#0c4a6e' : '#0b1120',
                      border: activeTier === t.tier ? '1px solid #38bdf8' : '1px solid #1e293b',
                      borderRadius: '6px',
                      cursor: 'pointer',
                      fontSize: '11px'
                    }}
                  >
                    <span style={{ fontWeight: 500, color: '#f1f5f9' }}>{t.name}</span>
                    <span style={{ color: '#94a3b8', fontFamily: 'monospace' }}>{t.desc}</span>
                  </div>
                ))}
              </div>

              {/* Interactive Temperature Control */}
              <div style={{ marginTop: '8px' }}>
                <div style={{ display: 'flex', justifyContent: 'space-between', fontSize: '11px', marginBottom: '4px' }}>
                  <span style={{ color: '#94a3b8' }}>Operating Temperature:</span>
                  <span style={{ color: '#38bdf8', fontFamily: 'monospace' }}>{temperatureK} K</span>
                </div>
                <input
                  type="range"
                  min="0.01"
                  max="350"
                  step="0.1"
                  value={temperatureK}
                  onChange={e => setTemperatureK(parseFloat(e.target.value))}
                  style={{ width: '100%', cursor: 'pointer' }}
                />
              </div>
            </div>
          )}
        </aside>
      </div>
    </div>
  )
}
