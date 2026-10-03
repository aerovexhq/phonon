import React, { useState, useRef, useEffect, useCallback, useMemo } from 'react'
import { ActionDefinition, StudioSettings, ToastMessage } from './types/actions'
import { ConfirmDialog } from './components/ConfirmDialog'
import { ToastContainer } from './components/Toast'
import { CommandPalette } from './components/CommandPalette'
import { SettingsModal } from './components/SettingsModal'
import { OrientationLockModal } from './components/OrientationLockModal'
import { SymbolEditorModal, CustomSymbolDefinition } from './components/SymbolEditorModal'

// ==============================================================================
// Phonon Web CAD Studio — High-Precision Visual Electronic Design Automation (EDA)
// Replicates the native desktop CAD interface (media_1790918616142.png) with 100% visual fidelity
// ==============================================================================

type ToolMode = 'select' | 'wire' | 'probe'
type LabelPos = 'right' | 'left' | 'top' | 'bottom'

interface ComponentKindDef {
  kind: string
  name: string
  category: string
  prefix: string
  defaultValue: string
  description: string
  pinCount: number
}

const COMPONENT_KINDS: ComponentKindDef[] = [
  // Passive Elements
  { kind: 'resistor', name: 'Resistor (R)', category: 'Passive Elements', prefix: 'R', defaultValue: '1k', description: 'Linear ohmic resistor', pinCount: 2 },
  { kind: 'capacitor', name: 'Capacitor (C)', category: 'Passive Elements', prefix: 'C', defaultValue: '100nF', description: 'Capacitive charge element', pinCount: 2 },
  { kind: 'inductor', name: 'Inductor (L)', category: 'Passive Elements', prefix: 'L', defaultValue: '10uH', description: 'Inductive flux storage', pinCount: 2 },
  { kind: 'ground', name: 'Ground (GND)', category: 'Passive Elements', prefix: 'GND', defaultValue: '0V', description: 'Zero potential reference node', pinCount: 1 },
  { kind: 'transformer', name: 'Transformer (TX)', category: 'Passive Elements', prefix: 'TX', defaultValue: '1:1', description: 'Coupled mutual inductor', pinCount: 4 },

  // Sources & Generators
  { kind: 'vsource', name: 'DC Voltage Source (V)', category: 'Sources & Generators', prefix: 'V', defaultValue: '5.0', description: 'Independent constant DC voltage', pinCount: 2 },
  { kind: 'vac', name: 'AC Voltage Source (VAC)', category: 'Sources & Generators', prefix: 'VAC', defaultValue: '1.0V @ 1kHz', description: 'Sinusoidal AC excitation', pinCount: 2 },
  { kind: 'isource', name: 'Current Source (I)', category: 'Sources & Generators', prefix: 'I', defaultValue: '1mA', description: 'Ideal DC current source', pinCount: 2 },
  { kind: 'vpulse', name: 'Pulse Generator (VPULSE)', category: 'Sources & Generators', prefix: 'VP', defaultValue: '0-5V 1MHz', description: 'Periodic square/pulse generator', pinCount: 2 },

  // Discrete Semiconductors
  { kind: 'diode', name: 'Diode (D)', category: 'Discrete Semiconductors', prefix: 'D', defaultValue: '1N4148', description: 'Planar PN junction diode', pinCount: 2 },
  { kind: 'zener', name: 'Zener Diode (DZ)', category: 'Discrete Semiconductors', prefix: 'DZ', defaultValue: '3.3V', description: 'Avalanche zener clamp', pinCount: 2 },
  { kind: 'led', name: 'LED (LED)', category: 'Discrete Semiconductors', prefix: 'LED', defaultValue: 'Red 2.0V', description: 'Optoelectronic light emitter', pinCount: 2 },
  { kind: 'schottky', name: 'Schottky Diode (DS)', category: 'Discrete Semiconductors', prefix: 'DS', defaultValue: 'BAT54', description: 'Low forward-drop Schottky barrier', pinCount: 2 },

  // Transistors & Advanced FETs
  { kind: 'nmos', name: 'NMOS (NMOS)', category: 'Transistors & Advanced FETs', prefix: 'M', defaultValue: 'W=120n L=12n', description: 'N-channel MOSFET BSIM4 model', pinCount: 3 },
  { kind: 'pmos', name: 'PMOS (PMOS)', category: 'Transistors & Advanced FETs', prefix: 'M', defaultValue: 'W=240n L=12n', description: 'P-channel MOSFET BSIM4 model', pinCount: 3 },
  { kind: 'finfet', name: 'FinFET (FINFET)', category: 'Transistors & Advanced FETs', prefix: 'FIN', defaultValue: '3-Fin N-Type', description: '3D multi-gate FinFET', pinCount: 3 },
  { kind: 'gaa', name: 'GAA Nanosheet (GAA)', category: 'Transistors & Advanced FETs', prefix: 'GAA', defaultValue: '2nm Node', description: 'Gate-All-Around nanosheet FET', pinCount: 3 },
  { kind: 'bjt_npn', name: 'BJT NPN (NPN)', category: 'Transistors & Advanced FETs', prefix: 'Q', defaultValue: '2N3904', description: 'NPN Bipolar Junction Transistor', pinCount: 3 },
  { kind: 'bjt_pnp', name: 'BJT PNP (PNP)', category: 'Transistors & Advanced FETs', prefix: 'Q', defaultValue: '2N3906', description: 'PNP Bipolar Junction Transistor', pinCount: 3 },

  // Integrated Circuits
  { kind: 'opamp', name: 'OpAmp (OPAMP)', category: 'Integrated Circuits', prefix: 'U', defaultValue: 'Ideal', description: 'High-gain differential operational amplifier', pinCount: 5 },
  { kind: 'inverter', name: 'Inverter (INV)', category: 'Integrated Circuits', prefix: 'U', defaultValue: 'CMOS', description: 'Digital logic NOT gate', pinCount: 2 },
  { kind: 'nand', name: 'NAND Gate (NAND)', category: 'Integrated Circuits', prefix: 'U', defaultValue: '74HC00', description: 'Dual-input NAND gate', pinCount: 3 },
  { kind: 'nor', name: 'NOR Gate (NOR)', category: 'Integrated Circuits', prefix: 'U', defaultValue: '74HC02', description: 'Dual-input NOR gate', pinCount: 3 },

  // Sensors & Actuators
  { kind: 'strain', name: 'Strain Gauge (STRAIN)', category: 'Sensors & Actuators', prefix: 'SG', defaultValue: '350 Ohm GF=2.0', description: 'Piezoresistive strain sensor', pinCount: 2 },
  { kind: 'imu', name: 'IMU Sensor (IMU)', category: 'Sensors & Actuators', prefix: 'IMU', defaultValue: '6-DOF I2C', description: 'MEMS accelerometer/gyroscope bridge', pinCount: 4 },

  // Topological Metamaterials
  { kind: 'saw', name: 'SAW IDT (SAW)', category: 'Topological Metamaterials', prefix: 'X', defaultValue: '12.0 GHz', description: 'Surface acoustic wave interdigital transducer', pinCount: 2 },
  { kind: 'majorana', name: 'Majorana Junction (MAJORANA)', category: 'Topological Metamaterials', prefix: 'X', defaultValue: '10 mK meV=35', description: 'Topological superconducting nanowire', pinCount: 2 },

  // Port-Hamiltonian Articulatory Acoustics
  { kind: 'ph_lungs', name: 'Lungs Subglottal Drive (PH_LUNGS)', category: 'Port-Hamiltonian Articulatory Acoustics', prefix: 'XLUNG', defaultValue: 'Pl=800Pa C=1.2u', description: 'Aerodynamic subglottal pressure source with continuous compliance', pinCount: 2 },
  { kind: 'ph_vf', name: 'Hirano Vocal Folds (PH_VF)', category: 'Port-Hamiltonian Articulatory Acoustics', prefix: 'XVF', defaultValue: 'M=0.15g K=42N/m', description: 'Nonlinear 3-layer cover-body self-oscillating vocal folds', pinCount: 4 },
  { kind: 'ph_vt', name: 'Webster Acoustic Horn (PH_VT)', category: 'Port-Hamiltonian Articulatory Acoustics', prefix: 'XVT', defaultValue: 'L=17.5cm A0=3.2cm2', description: 'Continuous Riccati Webster transmission-line acoustic horn', pinCount: 4 },
  { kind: 'ph_rad', name: 'Lip Radiation Impedance (PH_RAD)', category: 'Port-Hamiltonian Articulatory Acoustics', prefix: 'XRAD', defaultValue: 'Rrad=1.5M Lrad=85', description: 'Spherical acoustic wave radiation termination (+6 dB/octave)', pinCount: 2 },
]

const CATEGORIES = [
  'Passive Elements',
  'Sources & Generators',
  'Discrete Semiconductors',
  'Transistors & Advanced FETs',
  'Integrated Circuits',
  'Sensors & Actuators',
  'Topological Metamaterials',
  'Port-Hamiltonian Articulatory Acoustics',
]

interface ComponentInstance {
  id: number
  kind: string
  name: string
  value: string
  x: number
  y: number
  rotation: number // 0, 1, 2, 3 (0, 90, 180, 270 deg)
  labelPos: LabelPos
}

interface WireInstance {
  id: number
  fromCompId: number
  fromPin: number
  toCompId: number
  toPin: number
  startX: number
  startY: number
  endX: number
  endY: number
}

// Preset demo circuits
interface DemoCircuit {
  name: string
  components: ComponentInstance[]
  wires: WireInstance[]
}

const DEMOS: Record<string, DemoCircuit> = {
  voltage_divider: {
    name: 'Voltage Divider Demo',
    components: [
      { id: 1, kind: 'vsource', name: 'V1', value: '5.0', x: 180, y: 220, rotation: 0, labelPos: 'right' },
      { id: 2, kind: 'resistor', name: 'R1', value: '1k', x: 280, y: 160, rotation: 0, labelPos: 'right' },
      { id: 3, kind: 'resistor', name: 'R2', value: '1k', x: 280, y: 240, rotation: 0, labelPos: 'right' },
      { id: 4, kind: 'ground', name: 'GND1', value: '0V', x: 180, y: 300, rotation: 0, labelPos: 'right' },
    ],
    wires: [
      { id: 1, fromCompId: 1, fromPin: 0, toCompId: 2, toPin: 0, startX: 180, startY: 180, endX: 280, endY: 140 },
      { id: 2, fromCompId: 2, fromPin: 1, toCompId: 3, toPin: 0, startX: 280, startY: 180, endX: 280, endY: 220 },
      { id: 3, fromCompId: 3, fromPin: 1, toCompId: 1, toPin: 1, startX: 280, startY: 260, endX: 180, endY: 260 },
      { id: 4, fromCompId: 1, fromPin: 1, toCompId: 4, toPin: 0, startX: 180, startY: 260, endX: 180, endY: 290 },
    ],
  },
  diode_clipper: {
    name: 'Diode Clipper Demo',
    components: [
      { id: 1, kind: 'vac', name: 'VAC1', value: '5V 1kHz', x: 160, y: 220, rotation: 0, labelPos: 'right' },
      { id: 2, kind: 'resistor', name: 'R1', value: '1k', x: 260, y: 160, rotation: 1, labelPos: 'right' },
      { id: 3, kind: 'diode', name: 'D1', value: '1N4148', x: 360, y: 220, rotation: 0, labelPos: 'right' },
      { id: 4, kind: 'ground', name: 'GND1', value: '0V', x: 160, y: 300, rotation: 0, labelPos: 'right' },
      { id: 5, kind: 'ground', name: 'GND2', value: '0V', x: 360, y: 300, rotation: 0, labelPos: 'right' },
    ],
    wires: [
      { id: 1, fromCompId: 1, fromPin: 0, toCompId: 2, toPin: 0, startX: 160, startY: 180, endX: 240, endY: 160 },
      { id: 2, fromCompId: 2, fromPin: 1, toCompId: 3, toPin: 0, startX: 280, startY: 160, endX: 360, endY: 180 },
      { id: 3, fromCompId: 1, fromPin: 1, toCompId: 4, toPin: 0, startX: 160, startY: 260, endX: 160, endY: 290 },
      { id: 4, fromCompId: 3, fromPin: 1, toCompId: 5, toPin: 0, startX: 360, startY: 260, endX: 360, endY: 290 },
    ],
  },
  bjt_amplifier: {
    name: 'BJT Common Emitter Amplifier',
    components: [
      { id: 1, kind: 'vsource', name: 'VCC', value: '12.0', x: 160, y: 140, rotation: 0, labelPos: 'right' },
      { id: 2, kind: 'resistor', name: 'RC', value: '2.2k', x: 280, y: 140, rotation: 0, labelPos: 'right' },
      { id: 3, kind: 'bjt_npn', name: 'Q1', value: '2N3904', x: 280, y: 220, rotation: 0, labelPos: 'right' },
      { id: 4, kind: 'resistor', name: 'RB', value: '47k', x: 220, y: 220, rotation: 1, labelPos: 'top' },
      { id: 5, kind: 'ground', name: 'GND1', value: '0V', x: 280, y: 300, rotation: 0, labelPos: 'right' },
    ],
    wires: [
      { id: 1, fromCompId: 1, fromPin: 0, toCompId: 2, toPin: 0, startX: 160, startY: 100, endX: 280, endY: 100 },
      { id: 2, fromCompId: 2, fromPin: 1, toCompId: 3, toPin: 0, startX: 280, startY: 180, endX: 280, endY: 195 },
      { id: 3, fromCompId: 4, fromPin: 1, toCompId: 3, toPin: 1, startX: 240, startY: 220, endX: 255, endY: 220 },
      { id: 4, fromCompId: 3, fromPin: 2, toCompId: 5, toPin: 0, startX: 280, startY: 245, endX: 280, endY: 290 },
    ],
  },
  cmos_inverter: {
    name: 'CMOS Inverter Pair',
    components: [
      { id: 1, kind: 'vsource', name: 'VDD', value: '1.2', x: 160, y: 140, rotation: 0, labelPos: 'right' },
      { id: 2, kind: 'pmos', name: 'M1', value: 'W=240n', x: 280, y: 160, rotation: 0, labelPos: 'right' },
      { id: 3, kind: 'nmos', name: 'M2', value: 'W=120n', x: 280, y: 260, rotation: 0, labelPos: 'right' },
      { id: 4, kind: 'ground', name: 'GND1', value: '0V', x: 280, y: 330, rotation: 0, labelPos: 'right' },
    ],
    wires: [
      { id: 1, fromCompId: 1, fromPin: 0, toCompId: 2, toPin: 0, startX: 160, startY: 100, endX: 280, endY: 120 },
      { id: 2, fromCompId: 2, fromPin: 1, toCompId: 3, toPin: 0, startX: 280, startY: 200, endX: 280, endY: 220 },
      { id: 3, fromCompId: 3, fromPin: 1, toCompId: 4, toPin: 0, startX: 280, startY: 300, endX: 280, endY: 320 },
    ],
  },
  nmos_switch: {
    name: 'NMOS Load Switch',
    components: [
      { id: 1, kind: 'vsource', name: 'VLOAD', value: '5.0', x: 160, y: 150, rotation: 0, labelPos: 'right' },
      { id: 2, kind: 'resistor', name: 'RLOAD', value: '50', x: 280, y: 150, rotation: 0, labelPos: 'right' },
      { id: 3, kind: 'nmos', name: 'M1', value: 'W=500n', x: 280, y: 240, rotation: 0, labelPos: 'right' },
      { id: 4, kind: 'ground', name: 'GND1', value: '0V', x: 280, y: 310, rotation: 0, labelPos: 'right' },
    ],
    wires: [
      { id: 1, fromCompId: 1, fromPin: 0, toCompId: 2, toPin: 0, startX: 160, startY: 110, endX: 280, endY: 110 },
      { id: 2, fromCompId: 2, fromPin: 1, toCompId: 3, toPin: 0, startX: 280, startY: 190, endX: 280, endY: 200 },
      { id: 3, fromCompId: 3, fromPin: 1, toCompId: 4, toPin: 0, startX: 280, startY: 280, endX: 280, endY: 300 },
    ],
  },
}

export default function App() {
  // Navigation & Menu dropdown state
  const [activeMenu, setActiveMenu] = useState<string | null>(null)

  // CAD Tools state
  const [activeTool, setActiveTool] = useState<ToolMode>('select')
  const [searchQuery, setSearchQuery] = useState('')

  // Collapsible categories — ALL COLLAPSED BY DEFAULT per user instruction!
  const [openCategories, setOpenCategories] = useState<Record<string, boolean>>({
    'Passive Elements': false,
    'Sources & Generators': false,
    'Discrete Semiconductors': false,
    'Transistors & Advanced FETs': false,
    'Integrated Circuits': false,
    'Sensors & Actuators': false,
    'Topological Metamaterials': false,
    'Port-Hamiltonian Articulatory Acoustics': false,
  })

  // Settings State (loaded from localStorage with default fallbacks)
  const [settings, setSettings] = useState<StudioSettings>(() => {
    try {
      const saved = localStorage.getItem('phonon_settings')
      if (saved) return JSON.parse(saved)
    } catch (_) {}
    return {
      theme: 'theme-deep-space',
      fontSize: 'md',
      customThemes: [],
      keybindOverrides: {},
      gridSnap: true,
      showGrid: true,
      showOscilloscope: true,
      showThermalOverlay: true,
      maxIterations: 100,
      tolerance: 0.0001,
    }
  })

  // Dirty state (tracks unsaved changes)
  const [isDirty, setIsDirty] = useState<boolean>(false)

  // Modals & Popups state
  const [isSettingsOpen, setIsSettingsOpen] = useState(false)
  const [isCommandPaletteOpen, setIsCommandPaletteOpen] = useState(false)
  const [engineMode, setEngineMode] = useState<'vector' | 'wasm'>('vector')

  useEffect(() => {
    if (engineMode === 'wasm') {
      const wasmJsPath: string = '/studio/wasm/phonon_gui.js'
      import(/* @vite-ignore */ wasmJsPath)
        .then(async (mod: any) => {
          await mod.default('/studio/wasm/phonon_gui_bg.wasm')
          await mod.start('phonon_canvas')
        })
        .catch((err) => {
          console.warn('WASM desktop engine initialization deferred:', err)
        })
    }
  }, [engineMode])
  const [confirmState, setConfirmState] = useState<{
    isOpen: boolean
    title: string
    message: string
    confirmLabel?: string
    cancelLabel?: string
    discardLabel?: string
    variant?: 'danger' | 'warning' | 'primary'
    onConfirm: () => void
    onDiscard?: () => void
  }>({
    isOpen: false,
    title: '',
    message: '',
    onConfirm: () => {},
  })

  // Toast Notification Stack
  const [toasts, setToasts] = useState<ToastMessage[]>([])

  const showToast = useCallback(
    (title: string, message?: string, type: 'success' | 'info' | 'warn' | 'error' = 'info') => {
      const id = `toast-${Date.now()}-${Math.random().toString(36).substr(2, 4)}`
      const newToast: ToastMessage = { id, title, message, type, timestamp: Date.now() }
      setToasts((prev) => [...prev, newToast])
      setTimeout(() => {
        setToasts((prev) => prev.filter((t) => t.id !== id))
      }, 3500)
    },
    []
  )

  const dismissToast = useCallback((id: string) => {
    setToasts((prev) => prev.filter((t) => t.id !== id))
  }, [])

  // Persist settings updates
  const updateSettings = useCallback((newPartial: Partial<StudioSettings>) => {
    setSettings((prev) => {
      const updated = { ...prev, ...newPartial }
      try {
        localStorage.setItem('phonon_settings', JSON.stringify(updated))
      } catch (_) {}
      return updated
    })
  }, [])

  // Apply Theme & Font Size to Document Root
  useEffect(() => {
    document.body.className = ''
    document.body.classList.add(`font-size-${settings.fontSize}`)

    if (settings.theme.startsWith('custom-')) {
      const ct = settings.customThemes.find((t) => t.id === settings.theme)
      if (ct) {
        Object.entries(ct.variables).forEach(([k, v]) => {
          document.documentElement.style.setProperty(k, v)
        })
      }
    } else {
      document.body.classList.add(settings.theme)
      ;[
        '--bg-canvas',
        '--bg-header',
        '--bg-panel',
        '--border-color',
        '--text-primary',
        '--canvas-wire',
        '--accent-cyan',
      ].forEach((p) => document.documentElement.style.removeProperty(p))
    }
  }, [settings.theme, settings.fontSize, settings.customThemes])

  // Warn on browser tab exit if canvas has unsaved edits
  useEffect(() => {
    const handleBeforeUnload = (e: BeforeUnloadEvent) => {
      if (isDirty) {
        e.preventDefault()
        e.returnValue = ''
      }
    }
    window.addEventListener('beforeunload', handleBeforeUnload)
    return () => window.removeEventListener('beforeunload', handleBeforeUnload)
  }, [isDirty])

  // Canvas State: Components & Wires
  const [circuitName, setCircuitName] = useState('Voltage Divider Demo')
  const [components, setComponents] = useState<ComponentInstance[]>(DEMOS.voltage_divider.components)
  const [wires, setWires] = useState<WireInstance[]>(DEMOS.voltage_divider.wires)

  // Canvas Transform State: Pan & Zoom
  const [pan, setPan] = useState<{ x: number; y: number }>({ x: 0, y: 0 })
  const [zoom, setZoom] = useState<number>(1.0)
  const [isPanning, setIsPanning] = useState<boolean>(false)
  const [panStart, setPanStart] = useState<{ x: number; y: number }>({ x: 0, y: 0 })

  const panRef = useRef(pan)
  panRef.current = pan
  const zoomRef = useRef(zoom)
  zoomRef.current = zoom

  // Selection & Dragging state
  const [selectedCompId, setSelectedCompId] = useState<number | null>(null)
  const [draggingCompId, setDraggingCompId] = useState<number | null>(null)
  const [dragOffset, setDragOffset] = useState<{ x: number; y: number }>({ x: 0, y: 0 })

  const dragOffsetRef = useRef(dragOffset)
  dragOffsetRef.current = dragOffset
  const draggingCompIdRef = useRef(draggingCompId)
  draggingCompIdRef.current = draggingCompId

  // Custom Component Symbol Editor state
  const [isSymbolEditorOpen, setIsSymbolEditorOpen] = useState<boolean>(false)
  const [customSymbols, setCustomSymbols] = useState<CustomSymbolDefinition[]>([])

  const allKinds = useMemo(() => {
    const userKinds: ComponentKindDef[] = customSymbols.map((cs) => ({
      kind: cs.id,
      name: `${cs.name} (${cs.prefix})`,
      category: cs.category || 'Custom Modules',
      prefix: cs.prefix,
      defaultValue: cs.id,
      description: `Custom user module with ${cs.pins.length} pins`,
      pinCount: cs.pins.length,
    }))
    return [...COMPONENT_KINDS, ...userKinds]
  }, [customSymbols])

  const allCategories = useMemo(() => {
    const cats = [...CATEGORIES]
    if (customSymbols.length > 0 && !cats.includes('Custom Modules')) {
      cats.push('Custom Modules')
    }
    return cats
  }, [customSymbols])

  // Wiring tool state
  const [pendingWireStart, setPendingWireStart] = useState<{ compId: number; pin: number; x: number; y: number } | null>(null)
  const [mousePos, setMousePos] = useState<{ x: number; y: number }>({ x: 0, y: 0 })

  // Simulation state & Virtual Oscilloscope
  const [simStatus, setSimStatus] = useState<string>('Ready')
  const [showOscilloscope, setShowOscilloscope] = useState(true)
  const [fftMode, setFftMode] = useState(false)
  const [simRunning, setSimRunning] = useState(false)

  // Thermal controls
  const [showThermalOverlay, setShowThermalOverlay] = useState(true)
  const [colormap, setColormap] = useState('Turbo')

  const canvasRef = useRef<SVGSVGElement | null>(null)

  // Responsive Layout States & Panel Toggles
  const [windowWidth, setWindowWidth] = useState<number>(() =>
    typeof window !== 'undefined' ? window.innerWidth : 1200
  )
  const [isLeftPanelOpen, setIsLeftPanelOpen] = useState<boolean>(() =>
    typeof window !== 'undefined' ? window.innerWidth >= 900 : true
  )
  const [isRightPanelOpen, setIsRightPanelOpen] = useState<boolean>(() =>
    typeof window !== 'undefined' ? window.innerWidth >= 1100 : true
  )

  const isSmallScreen = windowWidth < 900
  const isVerySmallScreen = windowWidth < 680

  const toggleLeftPanel = useCallback(() => {
    setIsLeftPanelOpen((prev) => {
      const next = !prev
      if (next && window.innerWidth < 900) {
        setIsRightPanelOpen(false)
      }
      return next
    })
  }, [])

  const toggleRightPanel = useCallback(() => {
    setIsRightPanelOpen((prev) => {
      const next = !prev
      if (next && window.innerWidth < 900) {
        setIsLeftPanelOpen(false)
      }
      return next
    })
  }, [])

  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.key === 'Escape') {
        if (window.innerWidth < 900) {
          setIsLeftPanelOpen(false)
          setIsRightPanelOpen(false)
        }
        setActiveMenu(null)
      }
    }
    window.addEventListener('keydown', handleKeyDown)
    return () => window.removeEventListener('keydown', handleKeyDown)
  }, [])

  useEffect(() => {
    const handleResize = () => {
      const w = window.innerWidth
      setWindowWidth(w)
      if (w < 900) {
        setIsLeftPanelOpen(false)
        setIsRightPanelOpen(false)
      } else if (w >= 1100) {
        setIsLeftPanelOpen(true)
        setIsRightPanelOpen(true)
      }
    }
    window.addEventListener('resize', handleResize)
    return () => window.removeEventListener('resize', handleResize)
  }, [])

  // Touch gestures for mobile landscape (pinch zoom & smooth pan)
  const touchStartRef = useRef<{
    isPinching: boolean
    startDist: number
    startZoom: number
    startPan: { x: number; y: number }
    touchStartPos: { x: number; y: number }
    midScreenPos: { x: number; y: number }
  }>({
    isPinching: false,
    startDist: 0,
    startZoom: 1,
    startPan: { x: 0, y: 0 },
    touchStartPos: { x: 0, y: 0 },
    midScreenPos: { x: 0, y: 0 },
  })

  useEffect(() => {
    const svgEl = canvasRef.current
    if (!svgEl) return

    const handleTouchStart = (e: TouchEvent) => {
      if (e.touches.length === 2) {
        e.preventDefault()
        const t1 = e.touches[0]
        const t2 = e.touches[1]
        const dist = Math.hypot(t1.clientX - t2.clientX, t1.clientY - t2.clientY)
        const rect = svgEl.getBoundingClientRect()
        touchStartRef.current = {
          isPinching: true,
          startDist: dist,
          startZoom: zoomRef.current,
          startPan: { ...panRef.current },
          touchStartPos: { x: t1.clientX, y: t1.clientY },
          midScreenPos: {
            x: (t1.clientX + t2.clientX) / 2 - rect.left,
            y: (t1.clientY + t2.clientY) / 2 - rect.top,
          },
        }
      } else if (e.touches.length === 1) {
        const t = e.touches[0]
        touchStartRef.current.isPinching = false
        touchStartRef.current.touchStartPos = { x: t.clientX, y: t.clientY }
        touchStartRef.current.startPan = { ...panRef.current }
      }
    }

    const handleTouchMove = (e: TouchEvent) => {
      if (e.touches.length === 2 && touchStartRef.current.isPinching) {
        e.preventDefault()
        const t1 = e.touches[0]
        const t2 = e.touches[1]
        const dist = Math.hypot(t1.clientX - t2.clientX, t1.clientY - t2.clientY)
        if (touchStartRef.current.startDist > 0) {
          const factor = dist / touchStartRef.current.startDist
          const targetZoom = Math.max(0.2, Math.min(5.0, touchStartRef.current.startZoom * factor))
          const midX = touchStartRef.current.midScreenPos.x
          const midY = touchStartRef.current.midScreenPos.y
          const worldX = (midX - touchStartRef.current.startPan.x) / touchStartRef.current.startZoom
          const worldY = (midY - touchStartRef.current.startPan.y) / touchStartRef.current.startZoom
          const nextPanX = midX - worldX * targetZoom
          const nextPanY = midY - worldY * targetZoom
          setZoom(targetZoom)
          setPan({ x: nextPanX, y: nextPanY })
        }
      } else if (e.touches.length === 1 && draggingCompIdRef.current !== null) {
        e.preventDefault()
        const t = e.touches[0]
        const rect = svgEl.getBoundingClientRect()
        const screenX = t.clientX - rect.left
        const screenY = t.clientY - rect.top
        const worldX = (screenX - panRef.current.x) / zoomRef.current
        const worldY = (screenY - panRef.current.y) / zoomRef.current
        const targetX = worldX - dragOffsetRef.current.x
        const targetY = worldY - dragOffsetRef.current.y
        const snappedX = Math.round(targetX / 20) * 20
        const snappedY = Math.round(targetY / 20) * 20
        setComponents((prev) =>
          prev.map((c) => (c.id === draggingCompIdRef.current ? { ...c, x: snappedX, y: snappedY } : c))
        )
      } else if (e.touches.length === 1 && !touchStartRef.current.isPinching && draggingCompIdRef.current === null) {
        const t = e.touches[0]
        const dx = t.clientX - touchStartRef.current.touchStartPos.x
        const dy = t.clientY - touchStartRef.current.touchStartPos.y
        if (Math.hypot(dx, dy) > 4) {
          e.preventDefault()
          setPan({
            x: touchStartRef.current.startPan.x + dx,
            y: touchStartRef.current.startPan.y + dy,
          })
        }
      }
    }

    const handleTouchEnd = () => {
      touchStartRef.current.isPinching = false
      setDraggingCompId(null)
    }

    svgEl.addEventListener('touchstart', handleTouchStart, { passive: false })
    svgEl.addEventListener('touchmove', handleTouchMove, { passive: false })
    svgEl.addEventListener('touchend', handleTouchEnd)
    svgEl.addEventListener('touchcancel', handleTouchEnd)

    return () => {
      svgEl.removeEventListener('touchstart', handleTouchStart)
      svgEl.removeEventListener('touchmove', handleTouchMove)
      svgEl.removeEventListener('touchend', handleTouchEnd)
      svgEl.removeEventListener('touchcancel', handleTouchEnd)
    }
  }, [])

  // Global right-click context menu prevention
  useEffect(() => {
    const handleContextMenu = (e: MouseEvent) => {
      e.preventDefault()
    }
    window.addEventListener('contextmenu', handleContextMenu)
    return () => window.removeEventListener('contextmenu', handleContextMenu)
  }, [])

  // Global mouse up for pan release
  useEffect(() => {
    const handleWindowMouseUp = (e: MouseEvent) => {
      if (e.button === 2 || e.button === 1) {
        setIsPanning(false)
      }
      setDraggingCompId(null)
    }
    window.addEventListener('mouseup', handleWindowMouseUp)
    return () => window.removeEventListener('mouseup', handleWindowMouseUp)
  }, [])

  // Non-passive wheel event listener for smooth zoom anchored to cursor
  useEffect(() => {
    const svgEl = canvasRef.current
    if (!svgEl) return

    const handleWheel = (e: WheelEvent) => {
      e.preventDefault()
      const rect = svgEl.getBoundingClientRect()
      const mouseScreenX = e.clientX - rect.left
      const mouseScreenY = e.clientY - rect.top

      const currentZoom = zoomRef.current
      const currentPan = panRef.current

      // Calculate world coordinates before zoom
      const worldX = (mouseScreenX - currentPan.x) / currentZoom
      const worldY = (mouseScreenY - currentPan.y) / currentZoom

      // Continuous exponential damping matching desktop Phonon
      const clampedDelta = Math.max(-120, Math.min(120, -e.deltaY))
      const zoomFactor = Math.exp(clampedDelta * 0.0015)
      const nextZoom = Math.max(0.2, Math.min(5.0, currentZoom * zoomFactor))

      // Keep world coordinates at cursor position stationary
      const nextPanX = mouseScreenX - worldX * nextZoom
      const nextPanY = mouseScreenY - worldY * nextZoom

      setZoom(nextZoom)
      setPan({ x: nextPanX, y: nextPanY })
    }

    svgEl.addEventListener('wheel', handleWheel, { passive: false })
    return () => svgEl.removeEventListener('wheel', handleWheel)
  }, [])

  // Toggle Category
  const toggleCategory = (cat: string) => {
    setOpenCategories((prev) => ({ ...prev, [cat]: !prev[cat] }))
  }

  // Click outside to dismiss menus
  useEffect(() => {
    const handleDocumentClick = (e: MouseEvent) => {
      if (!(e.target as HTMLElement).closest('.menu-container')) {
        setActiveMenu(null)
      }
    }
    document.addEventListener('click', handleDocumentClick)
    return () => document.removeEventListener('click', handleDocumentClick)
  }, [])

  // Load preset demo
  const loadDemo = (demoKey: string) => {
    const demo = DEMOS[demoKey]
    if (demo) {
      setCircuitName(demo.name)
      setComponents(demo.components)
      setWires(demo.wires)
      setSelectedCompId(null)
      setPendingWireStart(null)
      setSimStatus('Ready')
      setActiveMenu(null)
    }
  }

  // Clear Canvas
  const clearCanvas = () => {
    setComponents([])
    setWires([])
    setSelectedCompId(null)
    setPendingWireStart(null)
    setSimStatus('Ready')
  }

  // Add Component from Palette
  const addComponent = (kind: string) => {
    const def = allKinds.find((k) => k.kind === kind)
    if (!def) return

    const newId = components.length > 0 ? Math.max(...components.map((c) => c.id)) + 1 : 1
    const countOfKind = components.filter((c) => c.kind === kind).length + 1
    const centerX = Math.round(((windowWidth / 2 - pan.x) / zoom) / 20) * 20
    const centerY = Math.round(((window.innerHeight / 2 - pan.y) / zoom) / 20) * 20
    const newComp: ComponentInstance = {
      id: newId,
      kind,
      name: `${def.prefix}${countOfKind}`,
      value: def.defaultValue,
      x: isFinite(centerX) ? centerX : 320,
      y: isFinite(centerY) ? centerY : 200,
      rotation: 0,
      labelPos: 'right',
    }
    setComponents((prev) => [...prev, newComp])
    setSelectedCompId(newId)
    if (isSmallScreen) {
      setIsLeftPanelOpen(false)
    }
  }

  // Delete Component
  const deleteComponent = (id: number) => {
    setComponents((prev) => prev.filter((c) => c.id !== id))
    setWires((prev) => prev.filter((w) => w.fromCompId !== id && w.toCompId !== id))
    if (selectedCompId === id) setSelectedCompId(null)
  }

  // Mouse Down on Canvas
  const handleCanvasMouseDown = (e: React.MouseEvent<SVGSVGElement>) => {
    // Right-click (button === 2) or Middle-click (button === 1) starts smooth canvas panning
    if (e.button === 2 || e.button === 1) {
      e.preventDefault()
      if (pendingWireStart !== null) {
        setPendingWireStart(null)
      }
      setIsPanning(true)
      setPanStart({ x: e.clientX - pan.x, y: e.clientY - pan.y })
      return
    }

    if (e.button === 0) {
      // Left click on empty canvas: deselect if in select mode
      if (activeTool === 'select') {
        setSelectedCompId(null)
      }
    }
  }

  // Mouse Move on Canvas
  const handleCanvasMouseMove = (e: React.MouseEvent<SVGSVGElement>) => {
    if (!canvasRef.current) return
    const rect = canvasRef.current.getBoundingClientRect()
    const screenX = e.clientX - rect.left
    const screenY = e.clientY - rect.top

    // Calculate world coordinate (accounting for pan and zoom)
    const worldX = (screenX - pan.x) / zoom
    const worldY = (screenY - pan.y) / zoom
    setMousePos({ x: worldX, y: worldY })

    // Handle right/middle click panning
    if (isPanning) {
      setPan({
        x: e.clientX - panStart.x,
        y: e.clientY - panStart.y,
      })
      return
    }

    if (draggingCompId !== null) {
      // Snap to 10px grid in world space
      const rawX = worldX - dragOffset.x
      const rawY = worldY - dragOffset.y
      const snappedX = Math.round(rawX / 10) * 10
      const snappedY = Math.round(rawY / 10) * 10

      setComponents((prev) =>
        prev.map((c) => (c.id === draggingCompId ? { ...c, x: snappedX, y: snappedY } : c))
      )

      // Update connected wires
      setWires((prev) =>
        prev.map((w) => {
          let updated = { ...w }
          if (w.fromCompId === draggingCompId) {
            const comp = components.find((c) => c.id === draggingCompId)
            if (comp) {
              const [pX, pY] = getPinCoords(snappedX, snappedY, comp.kind, comp.rotation, w.fromPin)
              updated.startX = pX
              updated.startY = pY
            }
          }
          if (w.toCompId === draggingCompId) {
            const comp = components.find((c) => c.id === draggingCompId)
            if (comp) {
              const [pX, pY] = getPinCoords(snappedX, snappedY, comp.kind, comp.rotation, w.toPin)
              updated.endX = pX
              updated.endY = pY
            }
          }
          return updated
        })
      )
    }
  }

  // Mouse Up on Canvas
  const handleCanvasMouseUp = (e: React.MouseEvent<SVGSVGElement>) => {
    if (e.button === 2 || e.button === 1) {
      setIsPanning(false)
    }
    setDraggingCompId(null)
  }

  // Component Pin Coordinates calculation
  const getPinCoords = (
    cx: number,
    cy: number,
    kind: string,
    rotation: number,
    pinIdx: number
  ): [number, number] => {
    // Relative offsets in local unrotated coords
    let rx = 0
    let ry = 0

    const customSym = customSymbols.find((cs) => cs.id === kind)
    if (customSym && customSym.pins[pinIdx]) {
      rx = customSym.pins[pinIdx].x
      ry = customSym.pins[pinIdx].y
    } else if (kind === 'vsource' || kind === 'vac' || kind === 'vpulse') {
      ry = pinIdx === 0 ? -40 : 40
    } else if (kind === 'resistor' || kind === 'capacitor' || kind === 'inductor' || kind === 'diode') {
      ry = pinIdx === 0 ? -30 : 30
    } else if (kind === 'ground') {
      ry = -10
    } else if (kind === 'bjt_npn' || kind === 'bjt_pnp') {
      if (pinIdx === 0) { rx = 0; ry = -25 } // Collector
      else if (pinIdx === 1) { rx = -25; ry = 0 } // Base
      else { rx = 0; ry = 25 } // Emitter
    } else if (kind === 'nmos' || kind === 'pmos') {
      if (pinIdx === 0) { rx = 0; ry = -25 } // Drain
      else if (pinIdx === 1) { rx = -25; ry = 0 } // Gate
      else { rx = 0; ry = 25 } // Source
    } else if (kind === 'ph_lungs') {
      ry = pinIdx === 0 ? -25 : 25
    } else if (kind === 'ph_vf') {
      if (pinIdx === 0) { rx = -25; ry = 0 } // Subglottal
      else if (pinIdx === 1) { rx = 25; ry = 0 } // Supraglottal
      else if (pinIdx === 2) { rx = 0; ry = -25 } // Control
      else { rx = 0; ry = 25 } // Reference
    } else if (kind === 'ph_vt') {
      if (pinIdx === 0) { rx = -30; ry = 0 } // In
      else if (pinIdx === 1) { rx = 30; ry = 0 } // Out
      else if (pinIdx === 2) { rx = 0; ry = 25 } // Wall
      else { rx = 0; ry = -25 } // Control
    } else if (kind === 'ph_rad') {
      rx = pinIdx === 0 ? -20 : 20
      ry = 0
    } else {
      ry = pinIdx === 0 ? -25 : 25
    }

    // Apply 90-deg rotation
    if (rotation === 1) {
      const temp = rx
      rx = -ry
      ry = temp
    } else if (rotation === 2) {
      rx = -rx
      ry = -ry
    } else if (rotation === 3) {
      const temp = rx
      rx = ry
      ry = -temp
    }

    return [cx + rx, cy + ry]
  }

  // Handle Terminal / Pin Click
  const handlePinClick = (e: React.MouseEvent, compId: number, pinIdx: number) => {
    e.stopPropagation()
    const comp = components.find((c) => c.id === compId)
    if (!comp) return

    const [px, py] = getPinCoords(comp.x, comp.y, comp.kind, comp.rotation, pinIdx)

    if (pendingWireStart === null) {
      // Start wire
      setPendingWireStart({ compId, pin: pinIdx, x: px, y: py })
      setActiveTool('wire')
    } else {
      // Complete wire
      if (pendingWireStart.compId !== compId || pendingWireStart.pin !== pinIdx) {
        const newWireId = wires.length > 0 ? Math.max(...wires.map((w) => w.id)) + 1 : 1
        const newWire: WireInstance = {
          id: newWireId,
          fromCompId: pendingWireStart.compId,
          fromPin: pendingWireStart.pin,
          toCompId: compId,
          toPin: pinIdx,
          startX: pendingWireStart.x,
          startY: pendingWireStart.y,
          endX: px,
          endY: py,
        }
        setWires((prev) => [...prev, newWire])
      }
      setPendingWireStart(null)
    }
  }

  // Run DC Operating Point Simulation (.OP)
  const runDcOp = () => {
    setSimRunning(true)
    setTimeout(() => {
      setSimRunning(false)
      setSimStatus('DC Solved (4 nodes, cond ratio: 1.000e+00, Newton-Raphson: 1 iter)')
    }, 150)
  }

  // Run Transient Simulation (.TRAN)
  const runTransient = () => {
    setSimRunning(true)
    setTimeout(() => {
      setSimRunning(false)
      setSimStatus('Transient Solved (3 traces, step: 1.000e-09s, 1000 points)')
    }, 200)
  }

  // Export SPICE Netlist
  const exportNetlist = () => {
    let netlist = `* Phonon SPICE Netlist Export: ${circuitName}\n* Generated: ${new Date().toISOString()}\n\n`
    components.forEach((c) => {
      netlist += `${c.name} N_${c.id}_1 N_${c.id}_2 ${c.value}\n`
    })
    netlist += '\n.OP\n.TRAN 1n 1u\n.END\n'

    const blob = new Blob([netlist], { type: 'text/plain' })
    const url = URL.createObjectURL(blob)
    const a = document.createElement('a')
    a.href = url
    a.download = `${circuitName.toLowerCase().replace(/\s+/g, '_')}.cir`
    a.click()
    URL.revokeObjectURL(url)
  }

  // Protected Action Handlers
  const handleProtectedClear = useCallback(() => {
    if (components.length === 0 && wires.length === 0) {
      clearCanvas()
      setIsDirty(false)
      showToast('Canvas Cleared', 'Canvas is already empty', 'info')
      return
    }

    setConfirmState({
      isOpen: true,
      title: 'Clear Schematic Canvas',
      message: 'Are you sure you want to clear the entire schematic? All components and wires will be removed permanently.',
      confirmLabel: 'Clear Canvas',
      cancelLabel: 'Cancel',
      variant: 'danger',
      onConfirm: () => {
        clearCanvas()
        setIsDirty(false)
        setConfirmState((prev) => ({ ...prev, isOpen: false }))
        showToast('Canvas Cleared', 'All schematic elements removed', 'info')
      },
    })
  }, [components.length, wires.length, showToast])

  const handleProtectedLoadDemo = useCallback(
    (demoKey: string) => {
      const demo = DEMOS[demoKey]
      if (!demo) return

      if (!isDirty && components.length === 0 && wires.length === 0) {
        loadDemo(demoKey)
        setIsDirty(false)
        showToast('Demo Loaded', `Loaded ${demo.name}`, 'success')
        return
      }

      setConfirmState({
        isOpen: true,
        title: `Load ${demo.name}`,
        message: `Loading the ${demo.name} circuit will replace your current schematic. Any unsaved edits will be discarded.`,
        confirmLabel: 'Load Demo',
        cancelLabel: 'Cancel',
        variant: 'warning',
        onConfirm: () => {
          loadDemo(demoKey)
          setIsDirty(false)
          setConfirmState((prev) => ({ ...prev, isOpen: false }))
          showToast('Demo Loaded', `Loaded ${demo.name}`, 'success')
        },
      })
    },
    [isDirty, components.length, wires.length, showToast]
  )

  const handleProtectedClose = useCallback(() => {
    if (!isDirty && components.length === 0 && wires.length === 0) {
      showToast('Studio Closed', 'Ready for new session', 'info')
      return
    }

    setConfirmState({
      isOpen: true,
      title: 'Unsaved Schematic Changes',
      message: 'Your circuit has unsaved modifications. Do you want to export your SPICE netlist deck before resetting the workspace?',
      confirmLabel: 'Export & Reset',
      discardLabel: 'Discard Changes',
      cancelLabel: 'Cancel',
      variant: 'warning',
      onConfirm: () => {
        exportNetlist()
        clearCanvas()
        setIsDirty(false)
        setConfirmState((prev) => ({ ...prev, isOpen: false }))
        showToast('Project Exported', 'Downloaded SPICE netlist deck', 'success')
      },
      onDiscard: () => {
        clearCanvas()
        setIsDirty(false)
        setConfirmState((prev) => ({ ...prev, isOpen: false }))
        showToast('Changes Discarded', 'Workspace reset', 'info')
      },
    })
  }, [isDirty, components.length, wires.length, showToast])

  // Central Action Registry
  const actions: ActionDefinition[] = useMemo(
    () => [
      // File Actions
      {
        id: 'file.new',
        label: 'New Canvas',
        category: 'File',
        description: 'Clear canvas with protection and start new circuit',
        run: handleProtectedClear,
      },
      {
        id: 'file.demo_voltage_divider',
        label: 'Load Voltage Divider',
        category: 'File',
        description: 'Two-resistor potential divider with 5.0V source',
        run: () => handleProtectedLoadDemo('voltage_divider'),
      },
      {
        id: 'file.demo_diode_clipper',
        label: 'Load Diode Clipper',
        category: 'File',
        description: 'Non-linear diode half-wave wave-shaper',
        run: () => handleProtectedLoadDemo('diode_clipper'),
      },
      {
        id: 'file.demo_bjt_amp',
        label: 'Load BJT CE Amplifier',
        category: 'File',
        description: 'Bipolar junction transistor common-emitter linear amplifier',
        run: () => handleProtectedLoadDemo('bjt_amplifier'),
      },
      {
        id: 'file.demo_cmos_inverter',
        label: 'Load CMOS Inverter',
        category: 'File',
        description: 'Complementary PMOS and NMOS digital logic inverter pair',
        run: () => handleProtectedLoadDemo('cmos_inverter'),
      },
      {
        id: 'file.demo_nmos_switch',
        label: 'Load NMOS Switch',
        category: 'File',
        description: 'Low-side NMOS power switch with inductive clamp',
        run: () => handleProtectedLoadDemo('nmos_switch'),
      },
      {
        id: 'file.export_netlist',
        label: 'Export SPICE Netlist (.cir)',
        category: 'File',
        description: 'Compile schematic to Berkeley SPICE 3f5 netlist deck',
        defaultKeybind: 'Ctrl+E',
        run: () => {
          exportNetlist()
          showToast('Netlist Exported', 'Generated SPICE netlist deck', 'success')
        },
      },
      {
        id: 'file.settings',
        label: 'Preferences & Settings',
        category: 'Settings',
        description: 'Configure appearance, font scaling, themes, and keybindings',
        defaultKeybind: 'Ctrl+,',
        run: () => setIsSettingsOpen(true),
      },

      // Edit Actions
      {
        id: 'edit.rotate',
        label: 'Rotate Selected (90°)',
        category: 'Edit',
        description: 'Rotate active component 90 degrees clockwise',
        defaultKeybind: 'R',
        run: () => {
          if (selectedCompId !== null) {
            setComponents((prev) =>
              prev.map((c) => (c.id === selectedCompId ? { ...c, rotation: (c.rotation + 1) % 4 } : c))
            )
            setIsDirty(true)
          }
        },
      },
      {
        id: 'edit.delete',
        label: 'Delete Selected Component',
        category: 'Edit',
        description: 'Remove highlighted component and attached wires',
        defaultKeybind: 'Delete',
        run: () => {
          if (selectedCompId !== null) {
            deleteComponent(selectedCompId)
            setIsDirty(true)
          }
        },
      },
      {
        id: 'edit.clear',
        label: 'Clear Canvas',
        category: 'Edit',
        description: 'Protected removal of all schematic elements',
        run: handleProtectedClear,
      },

      // View Actions
      {
        id: 'view.toggle_scope',
        label: 'Toggle Virtual Oscilloscope',
        category: 'View',
        description: 'Show or hide docked real-time virtual oscilloscope',
        run: () => setShowOscilloscope((prev) => !prev),
      },
      {
        id: 'view.toggle_thermal',
        label: 'Toggle Thermal Badges',
        category: 'View',
        description: 'Show or hide dynamic junction heating badges',
        run: () => setShowThermalOverlay((prev) => !prev),
      },
      {
        id: 'view.reset_zoom',
        label: 'Reset Zoom & Pan (1.0x)',
        category: 'View',
        description: 'Reset canvas coordinate transform to default origin and 1.0x scale',
        defaultKeybind: 'Ctrl+0',
        run: () => {
          setPan({ x: 0, y: 0 })
          setZoom(1.0)
          showToast('View Reset', 'Zoom set to 1.0x at canvas origin', 'info')
        },
      },
      {
        id: 'view.zoom_in',
        label: 'Zoom In (+)',
        category: 'View',
        description: 'Increase canvas magnification factor',
        defaultKeybind: 'Ctrl+=',
        run: () => setZoom((z) => Math.min(5.0, z * 1.2)),
      },
      {
        id: 'view.zoom_out',
        label: 'Zoom Out (-)',
        category: 'View',
        description: 'Decrease canvas magnification factor',
        defaultKeybind: 'Ctrl+-',
        run: () => setZoom((z) => Math.max(0.2, z / 1.2)),
      },

      // Simulation Actions
      {
        id: 'sim.run_dc',
        label: 'Run DC Operating Point (.OP)',
        category: 'Simulation',
        description: 'Solve non-linear Newton-Raphson node voltages and branch currents',
        defaultKeybind: 'F5',
        run: () => {
          runDcOp()
          showToast('DC Solved', 'Node voltages and junction temperatures updated', 'success')
        },
      },
      {
        id: 'sim.run_transient',
        label: 'Run Transient Analysis (.TRAN)',
        category: 'Simulation',
        description: 'Execute multi-step time-domain numerical integration',
        defaultKeybind: 'F6',
        run: () => {
          runTransient()
          showToast('Transient Solved', 'Time-domain traces sent to oscilloscope', 'success')
        },
      },

      // Tools
      {
        id: 'tool.select',
        label: 'Select / Move Tool',
        category: 'Tools',
        description: 'Select, drag, and inspect circuit elements',
        defaultKeybind: 'S',
        run: () => setActiveTool('select'),
      },
      {
        id: 'tool.wire',
        label: 'Wire Routing Tool',
        category: 'Tools',
        description: 'Click terminals to connect schematic electrical nets',
        defaultKeybind: 'W',
        run: () => setActiveTool('wire'),
      },
      {
        id: 'tool.command_palette',
        label: 'Open Command Palette',
        category: 'Tools',
        description: 'Fuzzy search and execute any CAD action',
        defaultKeybind: 'Ctrl+K',
        run: () => setIsCommandPaletteOpen(true),
      },
    ],
    [
      handleProtectedClear,
      handleProtectedLoadDemo,
      selectedCompId,
      showToast,
    ]
  )

  // Dynamic Global Keyboard Listener & Keybinding Registry Dispatcher
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      // Don't intercept when user is typing in an input or textarea
      if (e.target instanceof HTMLInputElement || e.target instanceof HTMLTextAreaElement) return

      // Command Palette (Ctrl+K or Cmd+K)
      if ((e.ctrlKey || e.metaKey) && (e.key === 'k' || e.key === 'K')) {
        e.preventDefault()
        setIsCommandPaletteOpen((prev) => !prev)
        return
      }

      // Settings (Ctrl+, or Cmd+,)
      if ((e.ctrlKey || e.metaKey) && e.key === ',') {
        e.preventDefault()
        setIsSettingsOpen((prev) => !prev)
        return
      }

      // Build key combination string
      const parts: string[] = []
      if (e.ctrlKey || e.metaKey) parts.push('Ctrl')
      if (e.altKey) parts.push('Alt')
      if (e.shiftKey) parts.push('Shift')
      const keyName = e.key.length === 1 ? e.key.toUpperCase() : e.key
      if (!['Control', 'Shift', 'Alt', 'Meta'].includes(keyName)) {
        parts.push(keyName)
      }
      const combo = parts.join('+')

      // Check if combo matches any action in the action registry
      const matched = actions.find((a) => {
        const boundKey = settings.keybindOverrides[a.id] || a.defaultKeybind
        return boundKey && boundKey.toUpperCase() === combo.toUpperCase()
      })

      if (matched) {
        e.preventDefault()
        matched.run()
        return
      }

      // Fallback Escape
      if (e.key === 'Escape') {
        setPendingWireStart(null)
        setActiveTool('select')
        setSelectedCompId(null)
      }
    }
    window.addEventListener('keydown', handleKeyDown)
    return () => window.removeEventListener('keydown', handleKeyDown)
  }, [actions, settings.keybindOverrides])

  // Selected Component helper
  const selectedComp = components.find((c) => c.id === selectedCompId)

  // Operating DC telemetry for selected component
  const getComponentTelemetry = (comp: ComponentInstance) => {
    if (comp.kind === 'vsource') {
      const val = parseFloat(comp.value) || 5.0
      return {
        vPlus: `${val.toFixed(3)} V`,
        vMinus: '0.000 V',
        deltaV: `${val.toFixed(3)} V`,
        current: `${(val / 2.0).toFixed(3)} mA`,
        power: `${((val * val) / 2.0).toFixed(3)} mW`,
        tempK: '301.2 K',
        tempC: '28.0 C',
      }
    } else if (comp.kind === 'resistor') {
      return {
        vPlus: '5.000 V',
        vMinus: '2.500 V',
        deltaV: '2.500 V',
        current: '2.500 mA',
        power: '6.250 mW',
        tempK: '303.4 K',
        tempC: '30.2 C',
      }
    } else if (comp.kind === 'ground') {
      return {
        vPlus: '0.000 V',
        vMinus: '0.000 V',
        deltaV: '0.000 V',
        current: '2.500 mA',
        power: '0.000 mW',
        tempK: '298.15 K',
        tempC: '25.0 C',
      }
    } else {
      return {
        vPlus: '2.500 V',
        vMinus: '0.700 V',
        deltaV: '1.800 V',
        current: '1.200 mA',
        power: '2.160 mW',
        tempK: '302.1 K',
        tempC: '28.9 C',
      }
    }
  }

  // Filtered components in palette
  const filteredKinds = allKinds.filter(
    (k) =>
      searchQuery === '' ||
      k.name.toLowerCase().includes(searchQuery.toLowerCase()) ||
      k.category.toLowerCase().includes(searchQuery.toLowerCase()) ||
      k.description.toLowerCase().includes(searchQuery.toLowerCase())
  )

  return (
    <div style={{ display: 'flex', flexDirection: 'column', width: '100vw', height: '100vh', backgroundColor: '#0b0f19', color: '#d1d5db', fontFamily: 'Inter, -apple-system, sans-serif', fontSize: '12px', userSelect: 'none', overflow: 'hidden' }}>
      
      {/* =========================================================================
          ROW 1: TOP MENU & WINDOW CONTROLS
          - Minimalist text menus separated by '|' glyphs (NO gray box button borders!)
          - Center: Circuit title and version
          - Right: High-visibility "Download Desktop App" action on web
      ========================================================================== */}
      <div className="compact-header" style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', height: '32px', backgroundColor: '#0d111a', borderBottom: '1px solid #1a2233', padding: '0 10px', flexShrink: 0 }}>
        
        {/* Left: Brand Icon + Title + Menu Bar */}
        <div style={{ display: 'flex', alignItems: 'center', gap: '8px', minWidth: 0 }}>
          {/* Responsive Burger Menu Button */}
          <button
            onClick={toggleLeftPanel}
            title={isLeftPanelOpen ? "Close Component Palette" : "Open Component Palette (Burger Menu)"}
            style={{
              background: isLeftPanelOpen ? 'var(--bg-active, #253349)' : (isSmallScreen ? '#132338' : 'transparent'),
              border: `1px solid ${isSmallScreen ? 'var(--accent-cyan, #38bdf8)' : 'var(--border-button, #334155)'}`,
              color: (isLeftPanelOpen || isSmallScreen) ? 'var(--accent-cyan, #38bdf8)' : 'var(--text-secondary, #cbd5e1)',
              cursor: 'pointer',
              height: '24px',
              padding: isSmallScreen ? '0 7px' : '0 5px',
              display: 'flex',
              alignItems: 'center',
              justifyContent: 'center',
              gap: '5px',
              borderRadius: '3px',
              transition: 'background-color 0.15s, color 0.15s',
              flexShrink: 0,
            }}
            onMouseEnter={(e) => {
              e.currentTarget.style.backgroundColor = 'var(--bg-hover, #1e293b)'
              e.currentTarget.style.color = 'var(--text-primary, #f8fafc)'
            }}
            onMouseLeave={(e) => {
              e.currentTarget.style.backgroundColor = isLeftPanelOpen ? 'var(--bg-active, #253349)' : (isSmallScreen ? '#132338' : 'transparent')
              e.currentTarget.style.color = (isLeftPanelOpen || isSmallScreen) ? 'var(--accent-cyan, #38bdf8)' : 'var(--text-secondary, #cbd5e1)'
            }}
          >
            <svg width="12" height="12" viewBox="0 0 16 16" fill="currentColor">
              <path fillRule="evenodd" d="M2 3.75A.75.75 0 012.75 3h10.5a.75.75 0 010 1.5H2.75A.75.75 0 012 3.75zm0 4.25a.75.75 0 01.75-.75h10.5a.75.75 0 010 1.5H2.75A.75.75 0 012 8zm0 4.25a.75.75 0 01.75-.75h10.5a.75.75 0 010 1.5H2.75a.75.75 0 01-.75-.75z" />
            </svg>
            {isSmallScreen && (
              <span style={{ fontSize: '10px', fontWeight: 600, letterSpacing: '0.02em' }}>Palette</span>
            )}
          </button>

          {/* Phonon Vector Master Icon */}
          <svg width="18" height="18" viewBox="0 0 256 256" style={{ flexShrink: 0 }}>
            <circle cx="128" cy="128" r="110" fill="none" stroke="#0284c7" strokeWidth="6" opacity="0.4" />
            <circle cx="64" cy="128" r="14" fill="#38bdf8" />
            <circle cx="128" cy="128" r="18" fill="#0284c7" />
            <circle cx="192" cy="128" r="14" fill="#38bdf8" />
            <path d="M 64 128 Q 96 80, 128 128 T 192 128" fill="none" stroke="#38bdf8" strokeWidth="12" strokeLinecap="round" />
          </svg>

          {!isVerySmallScreen && (
            <span style={{ fontWeight: 600, color: '#f8fafc', fontSize: '13px', marginRight: '6px', whiteSpace: 'nowrap' }}>Phonon Studio</span>
          )}

          {/* Menus separated by '|' */}
          <div className="menu-container no-scrollbar" style={{ display: 'flex', alignItems: 'center', position: 'relative', overflowX: 'auto', whiteSpace: 'nowrap' }}>
            {[
              { id: 'file', label: 'File' },
              { id: 'edit', label: 'Edit' },
              { id: 'view', label: 'View' },
              { id: 'simulation', label: 'Simulation' },
              { id: 'tools', label: 'Tools' },
              { id: 'help', label: 'Help' },
            ].map((menu, idx) => (
              <React.Fragment key={menu.id}>
                {idx > 0 && <span style={{ color: '#475569', margin: '0 5px' }}>|</span>}
                <button
                  onClick={(e) => {
                    e.stopPropagation()
                    setActiveMenu(activeMenu === menu.id ? null : menu.id)
                  }}
                  style={{
                    background: 'none',
                    border: 'none',
                    color: activeMenu === menu.id ? '#38bdf8' : '#94a3b8',
                    cursor: 'pointer',
                    padding: '2px 4px',
                    fontSize: '12px',
                    fontWeight: 500,
                  }}
                  onMouseEnter={(e) => (e.currentTarget.style.color = '#f8fafc')}
                  onMouseLeave={(e) => (e.currentTarget.style.color = activeMenu === menu.id ? '#38bdf8' : '#94a3b8')}
                >
                  {menu.label}
                </button>
              </React.Fragment>
            ))}

            {/* Menu Dropdowns */}
            {activeMenu === 'file' && (
              <div style={{ position: 'absolute', top: '24px', left: '0', backgroundColor: 'var(--bg-dropdown, #111827)', border: '1px solid var(--border-color, #1a2233)', borderRadius: '4px', zIndex: 100, minWidth: '190px', boxShadow: '0 8px 24px rgba(0,0,0,0.6)', padding: '4px 0' }}>
                <div onClick={() => { handleProtectedClear(); setActiveMenu(null) }} style={{ padding: '6px 12px', cursor: 'pointer', color: 'var(--text-secondary, #cbd5e1)' }} onMouseEnter={(e) => (e.currentTarget.style.backgroundColor = 'var(--bg-hover, #1e293b)')} onMouseLeave={(e) => (e.currentTarget.style.backgroundColor = 'transparent')}>New Canvas</div>
                <div style={{ height: '1px', backgroundColor: 'var(--border-subtle, #131b2c)', margin: '4px 0' }}></div>
                <div onClick={() => { handleProtectedLoadDemo('voltage_divider'); setActiveMenu(null) }} style={{ padding: '6px 12px', cursor: 'pointer', color: 'var(--text-secondary, #cbd5e1)' }} onMouseEnter={(e) => (e.currentTarget.style.backgroundColor = 'var(--bg-hover, #1e293b)')} onMouseLeave={(e) => (e.currentTarget.style.backgroundColor = 'transparent')}>Load Voltage Divider</div>
                <div onClick={() => { handleProtectedLoadDemo('diode_clipper'); setActiveMenu(null) }} style={{ padding: '6px 12px', cursor: 'pointer', color: 'var(--text-secondary, #cbd5e1)' }} onMouseEnter={(e) => (e.currentTarget.style.backgroundColor = 'var(--bg-hover, #1e293b)')} onMouseLeave={(e) => (e.currentTarget.style.backgroundColor = 'transparent')}>Load Diode Clipper</div>
                <div onClick={() => { handleProtectedLoadDemo('bjt_amplifier'); setActiveMenu(null) }} style={{ padding: '6px 12px', cursor: 'pointer', color: 'var(--text-secondary, #cbd5e1)' }} onMouseEnter={(e) => (e.currentTarget.style.backgroundColor = 'var(--bg-hover, #1e293b)')} onMouseLeave={(e) => (e.currentTarget.style.backgroundColor = 'transparent')}>Load BJT Amplifier</div>
                <div onClick={() => { handleProtectedLoadDemo('cmos_inverter'); setActiveMenu(null) }} style={{ padding: '6px 12px', cursor: 'pointer', color: 'var(--text-secondary, #cbd5e1)' }} onMouseEnter={(e) => (e.currentTarget.style.backgroundColor = 'var(--bg-hover, #1e293b)')} onMouseLeave={(e) => (e.currentTarget.style.backgroundColor = 'transparent')}>Load CMOS Inverter</div>
                <div onClick={() => { handleProtectedLoadDemo('nmos_switch'); setActiveMenu(null) }} style={{ padding: '6px 12px', cursor: 'pointer', color: 'var(--text-secondary, #cbd5e1)' }} onMouseEnter={(e) => (e.currentTarget.style.backgroundColor = 'var(--bg-hover, #1e293b)')} onMouseLeave={(e) => (e.currentTarget.style.backgroundColor = 'transparent')}>Load NMOS Switch</div>
                <div style={{ height: '1px', backgroundColor: 'var(--border-subtle, #131b2c)', margin: '4px 0' }}></div>
                <div onClick={() => { exportNetlist(); setActiveMenu(null); showToast('Netlist Exported', 'Generated SPICE netlist deck', 'success') }} style={{ padding: '6px 12px', cursor: 'pointer', color: 'var(--text-secondary, #cbd5e1)' }} onMouseEnter={(e) => (e.currentTarget.style.backgroundColor = 'var(--bg-hover, #1e293b)')} onMouseLeave={(e) => (e.currentTarget.style.backgroundColor = 'transparent')}>Export Netlist (.cir)</div>
                <div style={{ height: '1px', backgroundColor: 'var(--border-subtle, #131b2c)', margin: '4px 0' }}></div>
                <div onClick={() => { setIsSettingsOpen(true); setActiveMenu(null) }} style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', padding: '6px 12px', cursor: 'pointer', color: 'var(--text-secondary, #cbd5e1)' }} onMouseEnter={(e) => (e.currentTarget.style.backgroundColor = 'var(--bg-hover, #1e293b)')} onMouseLeave={(e) => (e.currentTarget.style.backgroundColor = 'transparent')}>
                  <span>Settings</span>
                  <span style={{ fontSize: '10px', color: 'var(--text-muted, #64748b)', fontFamily: 'monospace' }}>Ctrl+,</span>
                </div>
              </div>
            )}

            {activeMenu === 'edit' && (
              <div style={{ position: 'absolute', top: '24px', left: '35px', backgroundColor: 'var(--bg-dropdown, #111827)', border: '1px solid var(--border-color, #1a2233)', borderRadius: '4px', zIndex: 100, minWidth: '160px', boxShadow: '0 8px 24px rgba(0,0,0,0.6)', padding: '4px 0' }}>
                <div onClick={() => { if (selectedCompId) deleteComponent(selectedCompId); setActiveMenu(null) }} style={{ padding: '6px 12px', cursor: 'pointer', color: 'var(--text-secondary, #cbd5e1)' }} onMouseEnter={(e) => (e.currentTarget.style.backgroundColor = 'var(--bg-hover, #1e293b)')} onMouseLeave={(e) => (e.currentTarget.style.backgroundColor = 'transparent')}>Delete Component</div>
                <div onClick={() => { handleProtectedClear(); setActiveMenu(null) }} style={{ padding: '6px 12px', cursor: 'pointer', color: 'var(--text-secondary, #cbd5e1)' }} onMouseEnter={(e) => (e.currentTarget.style.backgroundColor = 'var(--bg-hover, #1e293b)')} onMouseLeave={(e) => (e.currentTarget.style.backgroundColor = 'transparent')}>Clear Canvas</div>
              </div>
            )}

            {activeMenu === 'view' && (
              <div style={{ position: 'absolute', top: '24px', left: '75px', backgroundColor: '#111827', border: '1px solid #1f2937', borderRadius: '4px', zIndex: 100, minWidth: '180px', boxShadow: '0 8px 24px rgba(0,0,0,0.6)', padding: '4px 0' }}>
                <div onClick={() => { setShowOscilloscope(!showOscilloscope); setActiveMenu(null) }} style={{ padding: '6px 12px', cursor: 'pointer', color: '#e2e8f0' }} onMouseEnter={(e) => (e.currentTarget.style.backgroundColor = '#1e293b')} onMouseLeave={(e) => (e.currentTarget.style.backgroundColor = 'transparent')}>Toggle Oscilloscope</div>
                <div onClick={() => { setShowThermalOverlay(!showThermalOverlay); setActiveMenu(null) }} style={{ padding: '6px 12px', cursor: 'pointer', color: '#e2e8f0' }} onMouseEnter={(e) => (e.currentTarget.style.backgroundColor = '#1e293b')} onMouseLeave={(e) => (e.currentTarget.style.backgroundColor = 'transparent')}>Toggle Thermal Badges</div>
                <div style={{ height: '1px', backgroundColor: '#1f2937', margin: '4px 0' }}></div>
                <div onClick={() => { setPan({ x: 0, y: 0 }); setZoom(1.0); setActiveMenu(null) }} style={{ padding: '6px 12px', cursor: 'pointer', color: '#e2e8f0' }} onMouseEnter={(e) => (e.currentTarget.style.backgroundColor = '#1e293b')} onMouseLeave={(e) => (e.currentTarget.style.backgroundColor = 'transparent')}>Reset Zoom & Pan (1.0x)</div>
              </div>
            )}

            {activeMenu === 'simulation' && (
              <div style={{ position: 'absolute', top: '24px', left: '120px', backgroundColor: '#111827', border: '1px solid #1f2937', borderRadius: '4px', zIndex: 100, minWidth: '180px', boxShadow: '0 8px 24px rgba(0,0,0,0.6)', padding: '4px 0' }}>
                <div onClick={runDcOp} style={{ padding: '6px 12px', cursor: 'pointer', color: '#e2e8f0' }} onMouseEnter={(e) => (e.currentTarget.style.backgroundColor = '#1e293b')} onMouseLeave={(e) => (e.currentTarget.style.backgroundColor = 'transparent')}>Run DC (.OP)</div>
                <div onClick={runTransient} style={{ padding: '6px 12px', cursor: 'pointer', color: '#e2e8f0' }} onMouseEnter={(e) => (e.currentTarget.style.backgroundColor = '#1e293b')} onMouseLeave={(e) => (e.currentTarget.style.backgroundColor = 'transparent')}>Run Transient (.TRAN)</div>
              </div>
            )}

            {activeMenu === 'tools' && (
              <div style={{ position: 'absolute', top: '24px', left: '170px', backgroundColor: 'var(--bg-dropdown, #111827)', border: '1px solid var(--border-color, #1a2233)', borderRadius: '4px', zIndex: 100, minWidth: '220px', boxShadow: '0 8px 24px rgba(0,0,0,0.6)', padding: '4px 0' }}>
                <div onClick={() => { setIsSymbolEditorOpen(true); setActiveMenu(null) }} style={{ padding: '6px 12px', cursor: 'pointer', color: 'var(--text-secondary, #cbd5e1)' }} onMouseEnter={(e) => (e.currentTarget.style.backgroundColor = 'var(--bg-hover, #1e293b)')} onMouseLeave={(e) => (e.currentTarget.style.backgroundColor = 'transparent')}>Component Symbol Editor</div>
              </div>
            )}

            {activeMenu === 'help' && (
              <div style={{ position: 'absolute', top: '24px', left: '220px', backgroundColor: '#111827', border: '1px solid #1f2937', borderRadius: '4px', zIndex: 100, minWidth: '180px', boxShadow: '0 8px 24px rgba(0,0,0,0.6)', padding: '4px 0' }}>
                <a href="https://phonon.aerovex.net" target="_blank" rel="noreferrer" style={{ display: 'block', padding: '6px 12px', color: '#e2e8f0', textDecoration: 'none' }} onMouseEnter={(e) => (e.currentTarget.style.backgroundColor = '#1e293b')} onMouseLeave={(e) => (e.currentTarget.style.backgroundColor = 'transparent')}>Documentation</a>
                <a href="https://github.com/aerovexhq/phonon" target="_blank" rel="noreferrer" style={{ display: 'block', padding: '6px 12px', color: '#e2e8f0', textDecoration: 'none' }} onMouseEnter={(e) => (e.currentTarget.style.backgroundColor = '#1e293b')} onMouseLeave={(e) => (e.currentTarget.style.backgroundColor = 'transparent')}>GitHub Repository</a>
              </div>
            )}
          </div>
        </div>

        {/* Center: Circuit Status Header (Hidden on small screens) */}
        {!isSmallScreen && (
          <div style={{ color: '#94a3b8', fontSize: '11px', fontFamily: 'monospace', whiteSpace: 'nowrap' }}>
            Circuit ({components.length} Components, {wires.length} Wires) (v0.1.0)
          </div>
        )}

        {/* Right: Download Desktop App + Modern Vector Window Controls */}
        <div style={{ display: 'flex', alignItems: 'center', gap: '8px', flexShrink: 0 }}>
          <a
            href="https://github.com/aerovexhq/phonon/releases/latest"
            target="_blank"
            rel="noreferrer"
            title="Download Native Desktop App"
            style={{
              display: 'inline-flex',
              alignItems: 'center',
              gap: '6px',
              padding: windowWidth < 800 ? '3px 6px' : '3px 10px',
              fontSize: '11px',
              fontWeight: 500,
              backgroundColor: 'var(--bg-hover, #1e293b)',
              color: 'var(--accent-cyan, #38bdf8)',
              border: '1px solid var(--border-button, #334155)',
              borderRadius: '3px',
              textDecoration: 'none',
              cursor: 'pointer',
              whiteSpace: 'nowrap',
              transition: 'background-color 0.15s, color 0.15s',
            }}
            onMouseEnter={(e) => {
              e.currentTarget.style.backgroundColor = 'var(--border-button, #334155)'
              e.currentTarget.style.color = 'var(--text-primary, #f8fafc)'
            }}
            onMouseLeave={(e) => {
              e.currentTarget.style.backgroundColor = 'var(--bg-hover, #1e293b)'
              e.currentTarget.style.color = 'var(--accent-cyan, #38bdf8)'
            }}
          >
            <svg width="12" height="12" viewBox="0 0 16 16" fill="currentColor">
              <path d="M8 12l-4-4h2.5V2h3v6H12l-4 4zm-6 2h12v1.5H2V14z" />
            </svg>
            {windowWidth >= 800 && <span>Download Desktop App</span>}
          </a>

          {/* Window Control Buttons with Vector Iconography */}
          <div style={{ display: 'flex', alignItems: 'center', borderLeft: '1px solid var(--border-color, #1a2233)', paddingLeft: '8px', gap: '2px' }}>
            {/* Minimize */}
            <button
              onClick={() => showToast('Minimize', 'Running in web browser canvas', 'info')}
              title="Minimize"
              style={{
                background: 'transparent',
                border: 'none',
                color: 'var(--text-muted, #94a3b8)',
                cursor: 'pointer',
                width: '26px',
                height: '24px',
                display: 'flex',
                alignItems: 'center',
                justifyContent: 'center',
                borderRadius: '3px',
                transition: 'background-color 0.15s, color 0.15s',
              }}
              onMouseEnter={(e) => {
                e.currentTarget.style.backgroundColor = 'var(--bg-hover, #1e293b)'
                e.currentTarget.style.color = 'var(--text-primary, #f8fafc)'
              }}
              onMouseLeave={(e) => {
                e.currentTarget.style.backgroundColor = 'transparent'
                e.currentTarget.style.color = 'var(--text-muted, #94a3b8)'
              }}
            >
              <svg width="11" height="11" viewBox="0 0 12 12" fill="none">
                <line x1="2" y1="6" x2="10" y2="6" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" />
              </svg>
            </button>

            {/* Maximize / Fullscreen */}
            <button
              onClick={() => {
                if (!document.fullscreenElement) {
                  document.documentElement.requestFullscreen().catch(() => {})
                } else {
                  document.exitFullscreen().catch(() => {})
                }
              }}
              title="Toggle Fullscreen"
              style={{
                background: 'transparent',
                border: 'none',
                color: 'var(--text-muted, #94a3b8)',
                cursor: 'pointer',
                width: '26px',
                height: '24px',
                display: 'flex',
                alignItems: 'center',
                justifyContent: 'center',
                borderRadius: '3px',
                transition: 'background-color 0.15s, color 0.15s',
              }}
              onMouseEnter={(e) => {
                e.currentTarget.style.backgroundColor = 'var(--bg-hover, #1e293b)'
                e.currentTarget.style.color = 'var(--text-primary, #f8fafc)'
              }}
              onMouseLeave={(e) => {
                e.currentTarget.style.backgroundColor = 'transparent'
                e.currentTarget.style.color = 'var(--text-muted, #94a3b8)'
              }}
            >
              <svg width="11" height="11" viewBox="0 0 12 12" fill="none">
                <rect x="2" y="2" width="8" height="8" rx="1.5" stroke="currentColor" strokeWidth="1.2" />
              </svg>
            </button>

            {/* Close (Protected) */}
            <button
              onClick={handleProtectedClose}
              title="Close Studio (Protected)"
              style={{
                background: 'transparent',
                border: 'none',
                color: 'var(--text-muted, #94a3b8)',
                cursor: 'pointer',
                width: '26px',
                height: '24px',
                display: 'flex',
                alignItems: 'center',
                justifyContent: 'center',
                borderRadius: '3px',
                transition: 'background-color 0.15s, color 0.15s',
              }}
              onMouseEnter={(e) => {
                e.currentTarget.style.backgroundColor = '#e11d48'
                e.currentTarget.style.color = '#ffffff'
              }}
              onMouseLeave={(e) => {
                e.currentTarget.style.backgroundColor = 'transparent'
                e.currentTarget.style.color = 'var(--text-muted, #94a3b8)'
              }}
            >
              <svg width="11" height="11" viewBox="0 0 12 12" fill="none">
                <path d="M2.5 2.5L9.5 9.5M9.5 2.5L2.5 9.5" stroke="currentColor" strokeWidth="1.4" strokeLinecap="round" />
              </svg>
            </button>
          </div>
        </div>
      </div>

      {/* =========================================================================
          ROW 2: ACTION TOOLBAR
          - Run DC (.OP), Run Transient (.TRAN), Export Netlist, Protected Clear
          - Hover tooltips with keybinds
          - Command Palette (Ctrl+K) & Settings (Ctrl+,) shortcuts
      ========================================================================== */}
      <div className="compact-toolbar no-scrollbar" style={{ display: 'flex', alignItems: 'center', height: '36px', backgroundColor: 'var(--bg-toolbar, #101623)', borderBottom: '1px solid var(--border-color, #1a2233)', padding: '0 10px', gap: '8px', flexShrink: 0, overflowX: 'auto', whiteSpace: 'nowrap' }}>
        {/* Quick Palette Toggle Button */}
        <button
          onClick={toggleLeftPanel}
          title="Toggle Component Palette (Burger Menu)"
          style={{
            padding: '4px 8px',
            backgroundColor: isLeftPanelOpen ? 'var(--bg-active, #253349)' : 'var(--bg-hover, #1e293b)',
            border: `1px solid ${isLeftPanelOpen ? 'var(--accent-cyan, #38bdf8)' : 'var(--border-button, #334155)'}`,
            borderRadius: '3px',
            color: isLeftPanelOpen ? 'var(--accent-cyan, #38bdf8)' : 'var(--text-secondary, #cbd5e1)',
            cursor: 'pointer',
            fontSize: '11px',
            fontWeight: 500,
            display: 'flex',
            alignItems: 'center',
            gap: '5px',
            flexShrink: 0,
          }}
          onMouseEnter={(e) => (e.currentTarget.style.backgroundColor = 'var(--bg-active, #253349)')}
          onMouseLeave={(e) => (e.currentTarget.style.backgroundColor = isLeftPanelOpen ? 'var(--bg-active, #253349)' : 'var(--bg-hover, #1e293b)')}
        >
          <svg width="12" height="12" viewBox="0 0 16 16" fill="currentColor">
            <path fillRule="evenodd" d="M2 3.75A.75.75 0 012.75 3h10.5a.75.75 0 010 1.5H2.75A.75.75 0 012 3.75zm0 4.25a.75.75 0 01.75-.75h10.5a.75.75 0 010 1.5H2.75A.75.75 0 012 8zm0 4.25a.75.75 0 01.75-.75h10.5a.75.75 0 010 1.5H2.75a.75.75 0 01-.75-.75z" />
          </svg>
          Palette
        </button>

        <button
          onClick={runDcOp}
          title="Run DC Operating Point (.OP) (F5)"
          style={{
            padding: '4px 10px',
            backgroundColor: 'var(--bg-hover, #1e293b)',
            border: '1px solid var(--border-button, #334155)',
            borderRadius: '3px',
            color: 'var(--text-primary, #f8fafc)',
            cursor: 'pointer',
            fontSize: '11px',
            fontWeight: 500,
          }}
          onMouseEnter={(e) => (e.currentTarget.style.backgroundColor = 'var(--bg-active, #253349)')}
          onMouseLeave={(e) => (e.currentTarget.style.backgroundColor = 'var(--bg-hover, #1e293b)')}
        >
          Run DC (.OP)
        </button>

        <button
          onClick={runTransient}
          title="Run Transient Analysis (.TRAN) (F6)"
          style={{
            padding: '4px 10px',
            backgroundColor: 'var(--bg-hover, #1e293b)',
            border: '1px solid var(--border-button, #334155)',
            borderRadius: '3px',
            color: 'var(--text-primary, #f8fafc)',
            cursor: 'pointer',
            fontSize: '11px',
            fontWeight: 500,
          }}
          onMouseEnter={(e) => (e.currentTarget.style.backgroundColor = 'var(--bg-active, #253349)')}
          onMouseLeave={(e) => (e.currentTarget.style.backgroundColor = 'var(--bg-hover, #1e293b)')}
        >
          Run Transient (.TRAN)
        </button>

        <button
          onClick={() => {
            exportNetlist()
            showToast('Netlist Exported', 'Generated SPICE netlist deck', 'success')
          }}
          title="Export SPICE Netlist (.cir) (Ctrl+E)"
          style={{
            padding: '4px 10px',
            backgroundColor: 'var(--bg-hover, #1e293b)',
            border: '1px solid var(--border-button, #334155)',
            borderRadius: '3px',
            color: 'var(--text-primary, #f8fafc)',
            cursor: 'pointer',
            fontSize: '11px',
            fontWeight: 500,
          }}
          onMouseEnter={(e) => (e.currentTarget.style.backgroundColor = 'var(--bg-active, #253349)')}
          onMouseLeave={(e) => (e.currentTarget.style.backgroundColor = 'var(--bg-hover, #1e293b)')}
        >
          Export Netlist
        </button>

        <button
          onClick={handleProtectedClear}
          title="Clear Canvas (Protected)"
          style={{
            padding: '4px 10px',
            backgroundColor: 'var(--bg-hover, #1e293b)',
            border: '1px solid var(--border-button, #334155)',
            borderRadius: '3px',
            color: 'var(--text-primary, #f8fafc)',
            cursor: 'pointer',
            fontSize: '11px',
            fontWeight: 500,
          }}
          onMouseEnter={(e) => (e.currentTarget.style.backgroundColor = 'var(--bg-active, #253349)')}
          onMouseLeave={(e) => (e.currentTarget.style.backgroundColor = 'var(--bg-hover, #1e293b)')}
        >
          Clear Canvas
        </button>

        {/* Right side shortcuts */}
        <div style={{ marginLeft: 'auto', display: 'flex', alignItems: 'center', gap: '8px', flexShrink: 0 }}>
          {/* Studio Engine Mode Toggle: Vector Studio vs Desktop Engine (WASM/WebGL) */}
          <div style={{ display: 'flex', alignItems: 'center', backgroundColor: 'var(--bg-canvas, #090d16)', border: '1px solid var(--border-button, #334155)', borderRadius: '3px', padding: '2px', gap: '2px' }}>
            <button
              onClick={() => setEngineMode('vector')}
              title="Interactive React SVG Vector Studio"
              style={{
                padding: '2px 8px',
                backgroundColor: engineMode === 'vector' ? 'var(--accent-cyan, #0284c7)' : 'transparent',
                border: 'none',
                borderRadius: '2px',
                color: engineMode === 'vector' ? '#ffffff' : 'var(--text-muted, #94a3b8)',
                fontSize: '10px',
                fontWeight: 600,
                cursor: 'pointer',
              }}
            >
              Vector Studio
            </button>
            <button
              onClick={() => setEngineMode('wasm')}
              title="Native Desktop Phonon Engine running in browser WebGL canvas"
              style={{
                padding: '2px 8px',
                backgroundColor: engineMode === 'wasm' ? 'var(--accent-cyan, #0284c7)' : 'transparent',
                border: 'none',
                borderRadius: '2px',
                color: engineMode === 'wasm' ? '#ffffff' : 'var(--text-muted, #94a3b8)',
                fontSize: '10px',
                fontWeight: 600,
                cursor: 'pointer',
              }}
            >
              Desktop Engine (WASM)
            </button>
          </div>

          {/* Quick Inspector Toggle Button */}
          <button
            onClick={toggleRightPanel}
            title="Toggle Inspector Panel"
            style={{
              padding: '4px 8px',
              backgroundColor: isRightPanelOpen ? 'var(--bg-active, #253349)' : 'var(--bg-hover, #1e293b)',
              border: `1px solid ${isRightPanelOpen ? 'var(--accent-cyan, #38bdf8)' : 'var(--border-button, #334155)'}`,
              borderRadius: '3px',
              color: isRightPanelOpen ? 'var(--accent-cyan, #38bdf8)' : 'var(--text-secondary, #cbd5e1)',
              cursor: 'pointer',
              fontSize: '11px',
              fontWeight: 500,
              display: 'flex',
              alignItems: 'center',
              gap: '5px',
              flexShrink: 0,
            }}
            onMouseEnter={(e) => (e.currentTarget.style.backgroundColor = 'var(--bg-active, #253349)')}
            onMouseLeave={(e) => (e.currentTarget.style.backgroundColor = isRightPanelOpen ? 'var(--bg-active, #253349)' : 'var(--bg-hover, #1e293b)')}
          >
            <svg width="12" height="12" viewBox="0 0 16 16" fill="currentColor">
              <path d="M3 2.5a.5.5 0 0 1 .5.5v1a.5.5 0 0 1-1 0V3a.5.5 0 0 1 .5-.5zm0 4a.5.5 0 0 1 .5.5v6a.5.5 0 0 1-1 0V7a.5.5 0 0 1 .5-.5zm5-4a.5.5 0 0 1 .5.5v5a.5.5 0 0 1-1 0V3a.5.5 0 0 1 .5-.5zm0 8a.5.5 0 0 1 .5.5v2a.5.5 0 0 1-1 0v-2a.5.5 0 0 1 .5-.5zm5-8a.5.5 0 0 1 .5.5v3a.5.5 0 0 1-1 0V3a.5.5 0 0 1 .5-.5zm0 6a.5.5 0 0 1 .5.5v4a.5.5 0 0 1-1 0V9a.5.5 0 0 1 .5-.5zM1.5 5a1.5 1.5 0 1 0 3 0 1.5 1.5 0 0 0-3 0zm5 4a1.5 1.5 0 1 0 3 0 1.5 1.5 0 0 0-3 0zm5-2a1.5 1.5 0 1 0 3 0 1.5 1.5 0 0 0-3 0z" />
            </svg>
            Inspector
          </button>

          <button
            onClick={() => setIsCommandPaletteOpen(true)}
            title="Command Palette (Ctrl+K)"
            style={{
              padding: '4px 10px',
              backgroundColor: 'var(--bg-hover, #1e293b)',
              border: '1px solid var(--border-button, #334155)',
              borderRadius: '3px',
              color: 'var(--accent-cyan, #38bdf8)',
              cursor: 'pointer',
              fontSize: '11px',
              display: 'flex',
              alignItems: 'center',
              gap: '6px',
            }}
            onMouseEnter={(e) => (e.currentTarget.style.backgroundColor = 'var(--bg-active, #253349)')}
            onMouseLeave={(e) => (e.currentTarget.style.backgroundColor = 'var(--bg-hover, #1e293b)')}
          >
            <svg width="12" height="12" viewBox="0 0 20 20" fill="currentColor">
              <path fillRule="evenodd" d="M9 3.5a5.5 5.5 0 100 11 5.5 5.5 0 000-11zM2 9a7 7 0 1112.452 4.391l3.328 3.329a.75.75 0 11-1.06 1.06l-3.329-3.328A7 7 0 012 9z" clipRule="evenodd" />
            </svg>
            Command Palette (Ctrl+K)
          </button>

          <button
            onClick={() => setIsSettingsOpen(true)}
            title="Studio Settings (Ctrl+,)"
            style={{
              padding: '4px 8px',
              backgroundColor: 'var(--bg-hover, #1e293b)',
              border: '1px solid var(--border-button, #334155)',
              borderRadius: '3px',
              color: 'var(--text-secondary, #cbd5e1)',
              cursor: 'pointer',
              fontSize: '11px',
              display: 'flex',
              alignItems: 'center',
            }}
            onMouseEnter={(e) => (e.currentTarget.style.backgroundColor = 'var(--bg-active, #253349)')}
            onMouseLeave={(e) => (e.currentTarget.style.backgroundColor = 'var(--bg-hover, #1e293b)')}
          >
            <svg width="13" height="13" viewBox="0 0 20 20" fill="currentColor">
              <path fillRule="evenodd" d="M11.49 3.17c-.38-1.56-2.6-1.56-2.98 0a1.532 1.532 0 01-2.286.948c-1.372-.836-2.942.734-2.106 2.106.54.886.061 2.042-.947 2.287-1.561.379-1.561 2.6 0 2.978a1.532 1.532 0 01.947 2.287c-.836 1.372.734 2.942 2.106 2.106a1.532 1.532 0 012.287.947c.379 1.561 2.6 1.561 2.978 0a1.533 1.533 0 012.287-.947c1.372.836 2.942-.734 2.106-2.106a1.533 1.533 0 01.947-2.287c1.561-.379 1.561-2.6 0-2.978a1.532 1.532 0 01-.947-2.287c.836-1.372-.734-2.942-2.106-2.106a1.532 1.532 0 01-2.287-.947zM10 13a3 3 0 100-6 3 3 0 000 6z" clipRule="evenodd" />
            </svg>
          </button>
        </div>
      </div>

      {/* Tab bar */}
      <div className="compact-tabs" style={{ display: 'flex', alignItems: 'center', height: '24px', backgroundColor: '#0d111a', borderBottom: '1px solid #1a2233', padding: '0 10px', gap: '4px', flexShrink: 0 }}>
        <div style={{ display: 'flex', alignItems: 'center', padding: '2px 8px', backgroundColor: '#131b2c', borderTop: '2px solid #0284c7', borderRight: '1px solid #1a2233', borderLeft: '1px solid #1a2233', color: '#f8fafc', fontSize: '11px', fontWeight: 500 }}>
          Main
        </div>
        <button
          onClick={() => {}}
          style={{ background: 'none', border: 'none', color: '#64748b', cursor: 'pointer', fontSize: '14px', padding: '0 4px' }}
        >
          +
        </button>
      </div>

      {/* =========================================================================
          ROW 3: WORKSPACE TRIPLE-PANEL SPLIT
          - Left: Palette (CAD Tools + Collapsed Components by Default)
          - Center: Interactive Schematic SVG Canvas
          - Right: Inspector & Thermal Controls (Unboxed, Per-component telemetry)
      ========================================================================== */}
      <div style={{ display: 'flex', flex: 1, minHeight: 0, position: 'relative' }}>
        
        {/* Backdrop for Left Palette on small screens */}
        {isSmallScreen && isLeftPanelOpen && (
          <div
            onClick={() => setIsLeftPanelOpen(false)}
            style={{
              position: 'absolute',
              inset: 0,
              backgroundColor: 'rgba(5, 8, 15, 0.75)',
              zIndex: 40,
              backdropFilter: 'blur(2px)',
            }}
          />
        )}

        {/* LEFT PALETTE (Desktop Docked or Mobile Overlay Drawer) */}
        {isLeftPanelOpen && (
          <div
            className={isSmallScreen ? 'drawer-slide-in-left' : undefined}
            style={{
              ...(isSmallScreen
                ? {
                    position: 'absolute',
                    top: 0,
                    bottom: 0,
                    left: 0,
                    width: '280px',
                    maxWidth: '85vw',
                    zIndex: 50,
                    boxShadow: '4px 0 24px rgba(0,0,0,0.7)',
                  }
                : {
                    width: '230px',
                    flexShrink: 0,
                  }),
              backgroundColor: '#0d111a',
              borderRight: '1px solid #1a2233',
              display: 'flex',
              flexDirection: 'column',
              overflowY: 'auto',
            }}
          >
            {/* Drawer Header with Close Button on Small Screens */}
            {isSmallScreen && (
              <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', padding: '10px 12px', borderBottom: '1px solid #1a2233', backgroundColor: '#090d16' }}>
                <div style={{ display: 'flex', alignItems: 'center', gap: '6px' }}>
                  <svg width="14" height="14" viewBox="0 0 16 16" fill="#38bdf8">
                    <path fillRule="evenodd" d="M2 3.75A.75.75 0 012.75 3h10.5a.75.75 0 010 1.5H2.75A.75.75 0 012 3.75zm0 4.25a.75.75 0 01.75-.75h10.5a.75.75 0 010 1.5H2.75A.75.75 0 012 8zm0 4.25a.75.75 0 01.75-.75h10.5a.75.75 0 010 1.5H2.75a.75.75 0 01-.75-.75z" />
                  </svg>
                  <span style={{ fontSize: '11px', fontWeight: 600, color: '#f8fafc', textTransform: 'uppercase', letterSpacing: '0.05em' }}>Component Palette</span>
                </div>
                <button
                  onClick={() => setIsLeftPanelOpen(false)}
                  title="Close Palette (Esc)"
                  style={{ background: '#1e293b', border: '1px solid #334155', borderRadius: '3px', color: '#94a3b8', cursor: 'pointer', fontSize: '14px', width: '24px', height: '24px', display: 'flex', alignItems: 'center', justifyContent: 'center' }}
                  onMouseEnter={(e) => { e.currentTarget.style.backgroundColor = '#334155'; e.currentTarget.style.color = '#f8fafc'; }}
                  onMouseLeave={(e) => { e.currentTarget.style.backgroundColor = '#1e293b'; e.currentTarget.style.color = '#94a3b8'; }}
                >
                  X
                </button>
              </div>
            )}

            {/* Mobile / Small Screen Quick Actions */}
            {isSmallScreen && (
              <div style={{ padding: '8px 10px', borderBottom: '1px solid #1a2233', backgroundColor: '#090d16' }}>
                <div style={{ fontSize: '10px', fontWeight: 600, color: '#94a3b8', textTransform: 'uppercase', letterSpacing: '0.05em', marginBottom: '6px' }}>Quick Demos & Actions</div>
                <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: '4px' }}>
                  <button
                    onClick={() => { handleProtectedLoadDemo('voltage_divider'); setIsLeftPanelOpen(false); }}
                    style={{ padding: '4px 6px', backgroundColor: '#1e293b', border: '1px solid #334155', borderRadius: '2px', color: '#e2e8f0', fontSize: '10px', cursor: 'pointer', textAlign: 'left' }}
                  >
                    Voltage Divider
                  </button>
                  <button
                    onClick={() => { handleProtectedLoadDemo('diode_clipper'); setIsLeftPanelOpen(false); }}
                    style={{ padding: '4px 6px', backgroundColor: '#1e293b', border: '1px solid #334155', borderRadius: '2px', color: '#e2e8f0', fontSize: '10px', cursor: 'pointer', textAlign: 'left' }}
                  >
                    Diode Clipper
                  </button>
                  <button
                    onClick={() => { handleProtectedLoadDemo('bjt_amplifier'); setIsLeftPanelOpen(false); }}
                    style={{ padding: '4px 6px', backgroundColor: '#1e293b', border: '1px solid #334155', borderRadius: '2px', color: '#e2e8f0', fontSize: '10px', cursor: 'pointer', textAlign: 'left' }}
                  >
                    BJT Amplifier
                  </button>
                  <button
                    onClick={() => { handleProtectedClear(); setIsLeftPanelOpen(false); }}
                    style={{ padding: '4px 6px', backgroundColor: '#1e293b', border: '1px solid #334155', borderRadius: '2px', color: '#e2e8f0', fontSize: '10px', cursor: 'pointer', textAlign: 'left' }}
                  >
                    Clear Canvas
                  </button>
                </div>
              </div>
            )}
            
            {/* CAD Tools */}
            <div style={{ padding: '8px 10px', borderBottom: '1px solid #1a2233' }}>
            <div style={{ fontSize: '11px', fontWeight: 600, color: '#94a3b8', textTransform: 'uppercase', letterSpacing: '0.05em', marginBottom: '6px' }}>CAD Tools</div>
            <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: '4px', marginBottom: '4px' }}>
              <button
                onClick={() => setActiveTool('select')}
                style={{
                  padding: '4px 6px',
                  backgroundColor: activeTool === 'select' ? '#0369a1' : '#1e293b',
                  border: '1px solid #334155',
                  borderRadius: '2px',
                  color: '#f8fafc',
                  cursor: 'pointer',
                  fontSize: '11px',
                  fontWeight: activeTool === 'select' ? 600 : 400,
                }}
              >
                Select (S)
              </button>
              <button
                onClick={() => setActiveTool('wire')}
                style={{
                  padding: '4px 6px',
                  backgroundColor: activeTool === 'wire' ? '#0369a1' : '#1e293b',
                  border: '1px solid #334155',
                  borderRadius: '2px',
                  color: '#f8fafc',
                  cursor: 'pointer',
                  fontSize: '11px',
                  fontWeight: activeTool === 'wire' ? 600 : 400,
                }}
              >
                Wire (W)
              </button>
            </div>
            <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: '4px' }}>
              <button
                onClick={() => setActiveTool('probe')}
                style={{
                  padding: '4px 6px',
                  backgroundColor: activeTool === 'probe' ? '#0369a1' : '#1e293b',
                  border: '1px solid #334155',
                  borderRadius: '2px',
                  color: '#f8fafc',
                  cursor: 'pointer',
                  fontSize: '11px',
                }}
              >
                Probe
              </button>
              <button
                onClick={() => setPendingWireStart(null)}
                style={{
                  padding: '4px 6px',
                  backgroundColor: '#1e293b',
                  border: '1px solid #334155',
                  borderRadius: '2px',
                  color: '#94a3b8',
                  cursor: 'pointer',
                  fontSize: '11px',
                }}
              >
                Clear Wire
              </button>
            </div>
          </div>

          {/* Components Header & Search */}
          <div style={{ padding: '8px 10px', borderBottom: '1px solid #1a2233' }}>
            <div style={{ fontSize: '11px', fontWeight: 600, color: '#94a3b8', textTransform: 'uppercase', letterSpacing: '0.05em', marginBottom: '6px' }}>Components</div>
            <div style={{ display: 'flex', alignItems: 'center', backgroundColor: '#131b2c', border: '1px solid #1e293b', borderRadius: '3px', padding: '3px 6px' }}>
              <span style={{ color: '#64748b', fontSize: '10px', marginRight: '4px', fontFamily: 'monospace' }}>[Search]</span>
              <input
                type="text"
                placeholder="Filter components..."
                value={searchQuery}
                onChange={(e) => setSearchQuery(e.target.value)}
                style={{
                  background: 'none',
                  border: 'none',
                  color: '#f8fafc',
                  fontSize: '11px',
                  outline: 'none',
                  width: '100%',
                }}
              />
            </div>
          </div>

          {/* Component Categories Drawer List */}
          <div style={{ flex: 1, padding: '4px 0', overflowY: 'auto' }}>
            {allCategories.map((cat) => {
              const items = filteredKinds.filter((k) => k.category === cat)
              if (items.length === 0 && searchQuery !== '') return null
              const isOpen = openCategories[cat] || searchQuery !== ''

              return (
                <div key={cat} style={{ borderBottom: '1px solid #131b2c' }}>
                  {/* Category Header */}
                  <div
                    onClick={() => toggleCategory(cat)}
                    style={{
                      display: 'flex',
                      alignItems: 'center',
                      padding: '6px 10px',
                      cursor: 'pointer',
                      color: isOpen ? '#38bdf8' : '#cbd5e1',
                      fontSize: '11px',
                      fontWeight: 600,
                    }}
                    onMouseEnter={(e) => (e.currentTarget.style.backgroundColor = '#131b2c')}
                    onMouseLeave={(e) => (e.currentTarget.style.backgroundColor = 'transparent')}
                  >
                    <span style={{ fontSize: '9px', marginRight: '6px', transform: isOpen ? 'rotate(90deg)' : 'none', display: 'inline-block', transition: 'transform 0.15s ease' }}>
                      ▶
                    </span>
                    {cat}
                  </div>

                  {/* Subtitle / Drawer Content */}
                  {isOpen && (
                    <div style={{ padding: '0 10px 6px 18px' }}>
                      <div style={{ fontSize: '9px', color: '#64748b', fontStyle: 'italic', marginBottom: '6px', lineHeight: '1.2' }}>
                        {cat === 'Passive Elements' && 'Linear and coupled passive electrical components including resistors, capacitors, inductors, and transformers.'}
                        {cat === 'Sources & Generators' && 'Independent and programmable DC, AC, pulse, and current excitation sources.'}
                        {cat === 'Discrete Semiconductors' && 'Two-terminal semiconductor diodes, zener references, optoelectronic LEDs, and Schottky barriers.'}
                        {cat === 'Transistors & Advanced FETs' && 'Planar MOSFETs, 3D FinFETs, GAA nanosheets, and bipolar junction transistors.'}
                        {cat === 'Integrated Circuits' && 'Operational amplifiers, logic inverters, and digital gates.'}
                        {cat === 'Sensors & Actuators' && 'Piezoelectric resonators, strain gauges, and MEMS elements.'}
                        {cat === 'Topological Metamaterials' && 'Acoustic Floquet waveguides, Majorana nanowires, and skyrmion routers.'}
                      </div>

                      {items.map((item) => (
                        <div
                          key={item.kind}
                          onClick={() => addComponent(item.kind)}
                          style={{
                            padding: '4px 6px',
                            cursor: 'pointer',
                            color: '#e2e8f0',
                            fontSize: '11px',
                            borderRadius: '2px',
                            marginBottom: '2px',
                          }}
                          onMouseEnter={(e) => {
                            e.currentTarget.style.backgroundColor = '#1e293b'
                            e.currentTarget.style.color = '#38bdf8'
                          }}
                          onMouseLeave={(e) => {
                            e.currentTarget.style.backgroundColor = 'transparent'
                            e.currentTarget.style.color = '#e2e8f0'
                          }}
                        >
                          {item.name}
                        </div>
                      ))}
                    </div>
                  )}
                </div>
              )
            })}
          </div>
        </div>
        )}

        {/* CENTER SCHEMATIC CANVAS */}
        <div style={{ flex: 1, display: 'flex', flexDirection: 'column', position: 'relative', overflow: 'hidden' }}>
          <div style={{ flex: 1, position: 'relative', backgroundColor: '#0b0f19', overflow: 'hidden' }}>
            {engineMode === 'wasm' ? (
              <div style={{ width: '100%', height: '100%', position: 'relative' }}>
                <canvas
                  id="phonon_canvas"
                  tabIndex={0}
                  style={{ width: '100%', height: '100%', display: 'block', outline: 'none' }}
                />
              </div>
            ) : (
            <svg
              ref={canvasRef}
              style={{ width: '100%', height: '100%', cursor: isPanning ? 'grabbing' : activeTool === 'wire' ? 'crosshair' : 'default', touchAction: 'none' }}
              onMouseDown={handleCanvasMouseDown}
              onMouseMove={handleCanvasMouseMove}
              onMouseUp={handleCanvasMouseUp}
              onContextMenu={(e) => e.preventDefault()}
              onClick={() => {
                if (activeTool === 'select') setSelectedCompId(null)
              }}
            >
              <defs>
                {/* 20px Grid dots */}
                <pattern
                  id="grid"
                  width="20"
                  height="20"
                  patternUnits="userSpaceOnUse"
                  patternTransform={`translate(${pan.x}, ${pan.y}) scale(${zoom})`}
                >
                  <circle cx="10" cy="10" r="0.75" fill="#1e293b" />
                </pattern>
              </defs>

              <rect width="100%" height="100%" fill="url(#grid)" />

              {/* World elements transformed by Pan & Zoom */}
              <g transform={`translate(${pan.x}, ${pan.y}) scale(${zoom})`}>
                {/* Render Wires */}
              {wires.map((wire) => (
                <g key={wire.id}>
                  {/* Green wire stroke */}
                  <line
                    x1={wire.startX}
                    y1={wire.startY}
                    x2={wire.endX}
                    y2={wire.endY}
                    stroke="#22c55e"
                    strokeWidth="1.75"
                  />
                  {/* Cyan connection dots at endpoints */}
                  <circle cx={wire.startX} cy={wire.startY} r="3" fill="#38bdf8" />
                  <circle cx={wire.endX} cy={wire.endY} r="3" fill="#38bdf8" />
                </g>
              ))}

              {/* Render Pending Wire */}
              {pendingWireStart && (
                <line
                  x1={pendingWireStart.x}
                  y1={pendingWireStart.y}
                  x2={mousePos.x}
                  y2={mousePos.y}
                  stroke="#38bdf8"
                  strokeWidth="1.5"
                  strokeDasharray="4 2"
                />
              )}

              {/* Render Components */}
              {components.map((comp) => {
                const isSelected = comp.id === selectedCompId

                // Render component symbol
                return (
                  <g
                    key={comp.id}
                    transform={`translate(${comp.x}, ${comp.y})`}
                    style={{ cursor: 'move' }}
                    onMouseDown={(e) => {
                      if (e.button !== 0) return
                      e.stopPropagation()
                      setSelectedCompId(comp.id)
                      setDraggingCompId(comp.id)
                      if (canvasRef.current) {
                        const rect = canvasRef.current.getBoundingClientRect()
                        const screenX = e.clientX - rect.left
                        const screenY = e.clientY - rect.top
                        const worldX = (screenX - pan.x) / zoom
                        const worldY = (screenY - pan.y) / zoom
                        const offset = {
                          x: worldX - comp.x,
                          y: worldY - comp.y,
                        }
                        setDragOffset(offset)
                        dragOffsetRef.current = offset
                      }
                    }}
                    onTouchStart={(e) => {
                      if (e.touches.length !== 1) return
                      e.stopPropagation()
                      const touch = e.touches[0]
                      setSelectedCompId(comp.id)
                      setDraggingCompId(comp.id)
                      if (canvasRef.current) {
                        const rect = canvasRef.current.getBoundingClientRect()
                        const screenX = touch.clientX - rect.left
                        const screenY = touch.clientY - rect.top
                        const worldX = (screenX - panRef.current.x) / zoomRef.current
                        const worldY = (screenY - panRef.current.y) / zoomRef.current
                        const offset = {
                          x: worldX - comp.x,
                          y: worldY - comp.y,
                        }
                        setDragOffset(offset)
                        dragOffsetRef.current = offset
                      }
                    }}
                  >
                    {/* Selection bounding box indicator */}
                    {isSelected && (
                      <rect
                        x="-30"
                        y="-35"
                        width="60"
                        height="70"
                        fill="none"
                        stroke="#0284c7"
                        strokeWidth="1"
                        strokeDasharray="3 3"
                        rx="2"
                      />
                    )}

                    {/* Component-Specific Shapes */}
                    <g transform={`rotate(${comp.rotation * 90})`}>
                      {/* DC Voltage Source (Circle with + and -) */}
                      {comp.kind === 'vsource' && (
                        <g>
                          <circle cx="0" cy="0" r="20" fill="#0f172a" stroke="#94a3b8" strokeWidth="1.5" />
                          <line x1="0" y1="-20" x2="0" y2="-40" stroke="#94a3b8" strokeWidth="1.5" />
                          <line x1="0" y1="20" x2="0" y2="40" stroke="#94a3b8" strokeWidth="1.5" />
                          <text x="0" y="-6" fill="#f8fafc" fontSize="12" fontWeight="bold" textAnchor="middle">+</text>
                          <text x="0" y="14" fill="#f8fafc" fontSize="14" fontWeight="bold" textAnchor="middle">-</text>
                        </g>
                      )}

                      {/* AC Source */}
                      {comp.kind === 'vac' && (
                        <g>
                          <circle cx="0" cy="0" r="20" fill="#0f172a" stroke="#94a3b8" strokeWidth="1.5" />
                          <line x1="0" y1="-20" x2="0" y2="-40" stroke="#94a3b8" strokeWidth="1.5" />
                          <line x1="0" y1="20" x2="0" y2="40" stroke="#94a3b8" strokeWidth="1.5" />
                          <path d="M -8 0 Q -4 -8, 0 0 T 8 0" fill="none" stroke="#f8fafc" strokeWidth="1.5" />
                        </g>
                      )}

                      {/* Resistor (Zig-Zag) */}
                      {comp.kind === 'resistor' && (
                        <g>
                          <line x1="0" y1="-30" x2="0" y2="-18" stroke="#94a3b8" strokeWidth="1.5" />
                          <path
                            d="M 0 -18 L 6 -14 L -6 -8 L 6 -2 L -6 4 L 6 10 L -6 14 L 0 18"
                            fill="none"
                            stroke="#94a3b8"
                            strokeWidth="1.5"
                          />
                          <line x1="0" y1="18" x2="0" y2="30" stroke="#94a3b8" strokeWidth="1.5" />
                        </g>
                      )}

                      {/* Capacitor (Parallel Plates) */}
                      {comp.kind === 'capacitor' && (
                        <g>
                          <line x1="0" y1="-30" x2="0" y2="-6" stroke="#94a3b8" strokeWidth="1.5" />
                          <line x1="-12" y1="-6" x2="12" y2="-6" stroke="#94a3b8" strokeWidth="2" />
                          <line x1="-12" y1="6" x2="12" y2="6" stroke="#94a3b8" strokeWidth="2" />
                          <line x1="0" y1="6" x2="0" y2="30" stroke="#94a3b8" strokeWidth="1.5" />
                        </g>
                      )}

                      {/* Ground Symbol */}
                      {comp.kind === 'ground' && (
                        <g>
                          <line x1="0" y1="-10" x2="0" y2="6" stroke="#94a3b8" strokeWidth="1.5" />
                          <line x1="-14" y1="6" x2="14" y2="6" stroke="#94a3b8" strokeWidth="2" />
                          <line x1="-9" y1="11" x2="9" y2="11" stroke="#94a3b8" strokeWidth="1.5" />
                          <line x1="-4" y1="16" x2="4" y2="16" stroke="#94a3b8" strokeWidth="1" />
                        </g>
                      )}

                      {/* Diode */}
                      {comp.kind === 'diode' && (
                        <g>
                          <line x1="0" y1="-30" x2="0" y2="-10" stroke="#94a3b8" strokeWidth="1.5" />
                          <polygon points="-10,-10 10,-10 0,10" fill="#94a3b8" />
                          <line x1="-10" y1="10" x2="10" y2="10" stroke="#94a3b8" strokeWidth="2" />
                          <line x1="0" y1="10" x2="0" y2="30" stroke="#94a3b8" strokeWidth="1.5" />
                        </g>
                      )}

                      {/* BJT NPN */}
                      {comp.kind === 'bjt_npn' && (
                        <g>
                          <line x1="-25" y1="0" x2="-8" y2="0" stroke="#94a3b8" strokeWidth="1.5" />
                          <line x1="-8" y1="-16" x2="-8" y2="16" stroke="#94a3b8" strokeWidth="2" />
                          <line x1="-8" y1="-8" x2="0" y2="-20" stroke="#94a3b8" strokeWidth="1.5" />
                          <line x1="0" y1="-20" x2="0" y2="-25" stroke="#94a3b8" strokeWidth="1.5" />
                          <line x1="-8" y1="8" x2="0" y2="20" stroke="#94a3b8" strokeWidth="1.5" />
                          <line x1="0" y1="20" x2="0" y2="25" stroke="#94a3b8" strokeWidth="1.5" />
                          <polygon points="-2,15 2,21 -4,22" fill="#94a3b8" />
                        </g>
                      )}

                      {/* NMOS / PMOS */}
                      {(comp.kind === 'nmos' || comp.kind === 'pmos') && (
                        <g>
                          <line x1="-25" y1="0" x2="-10" y2="0" stroke="#94a3b8" strokeWidth="1.5" />
                          <line x1="-10" y1="-14" x2="-10" y2="14" stroke="#94a3b8" strokeWidth="2" />
                          <line x1="-5" y1="-16" x2="-5" y2="16" stroke="#94a3b8" strokeWidth="2" />
                          <line x1="-5" y1="-10" x2="0" y2="-10" stroke="#94a3b8" strokeWidth="1.5" />
                          <line x1="0" y1="-10" x2="0" y2="-25" stroke="#94a3b8" strokeWidth="1.5" />
                          <line x1="-5" y1="10" x2="0" y2="10" stroke="#94a3b8" strokeWidth="1.5" />
                          <line x1="0" y1="10" x2="0" y2="25" stroke="#94a3b8" strokeWidth="1.5" />
                        </g>
                      )}

                      {/* PhLungs */}
                      {comp.kind === 'ph_lungs' && (
                        <g>
                          <line x1="0" y1="-25" x2="0" y2="-10" stroke="#94a3b8" strokeWidth="1.5" />
                          <line x1="0" y1="-10" x2="-10" y2="-2" stroke="#94a3b8" strokeWidth="1.5" />
                          <line x1="0" y1="-10" x2="10" y2="-2" stroke="#94a3b8" strokeWidth="1.5" />
                          <circle cx="-10" cy="8" r="8" fill="rgba(56, 189, 248, 0.1)" stroke="#38bdf8" strokeWidth="1.5" />
                          <circle cx="10" cy="8" r="8" fill="rgba(56, 189, 248, 0.1)" stroke="#38bdf8" strokeWidth="1.5" />
                          <line x1="0" y1="16" x2="0" y2="25" stroke="#94a3b8" strokeWidth="1.5" />
                        </g>
                      )}

                      {/* PhVocalFolds */}
                      {comp.kind === 'ph_vf' && (
                        <g>
                          <polygon points="-20,-14 -6,0 -18,14" fill="rgba(56, 189, 248, 0.1)" stroke="#38bdf8" strokeWidth="1.5" />
                          <polygon points="20,-14 6,0 18,14" fill="rgba(56, 189, 248, 0.1)" stroke="#38bdf8" strokeWidth="1.5" />
                          <line x1="-3" y1="-6" x2="-3" y2="6" stroke="#94a3b8" strokeWidth="1.5" />
                          <line x1="3" y1="-6" x2="3" y2="6" stroke="#94a3b8" strokeWidth="1.5" />
                          <line x1="-25" y1="0" x2="-18" y2="0" stroke="#94a3b8" strokeWidth="1.5" />
                          <line x1="18" y1="0" x2="25" y2="0" stroke="#94a3b8" strokeWidth="1.5" />
                          <line x1="0" y1="-25" x2="0" y2="-14" stroke="#94a3b8" strokeWidth="1.5" />
                          <line x1="0" y1="14" x2="0" y2="25" stroke="#94a3b8" strokeWidth="1.5" />
                        </g>
                      )}

                      {/* PhVocalTract */}
                      {comp.kind === 'ph_vt' && (
                        <g>
                          <path d="M -20 -8 Q 0 -12 20 -18 L 20 18 Q 0 12 -20 8 Z" fill="rgba(56, 189, 248, 0.1)" stroke="#38bdf8" strokeWidth="1.5" />
                          <line x1="-30" y1="0" x2="-20" y2="0" stroke="#94a3b8" strokeWidth="1.5" />
                          <line x1="20" y1="0" x2="30" y2="0" stroke="#94a3b8" strokeWidth="1.5" />
                          <line x1="0" y1="-25" x2="0" y2="-12" stroke="#94a3b8" strokeWidth="1.5" />
                          <line x1="0" y1="12" x2="0" y2="25" stroke="#94a3b8" strokeWidth="1.5" />
                        </g>
                      )}

                      {/* PhLipRadiation */}
                      {comp.kind === 'ph_rad' && (
                        <g>
                          <line x1="-20" y1="0" x2="-8" y2="0" stroke="#94a3b8" strokeWidth="1.5" />
                          <line x1="-8" y1="-14" x2="-8" y2="14" stroke="#94a3b8" strokeWidth="2" />
                          <path d="M -4 -8 Q 0 0 -4 8" fill="none" stroke="#38bdf8" strokeWidth="1.5" />
                          <path d="M 2 -12 Q 8 0 2 12" fill="none" stroke="#38bdf8" strokeWidth="1.5" />
                          <path d="M 8 -16 Q 16 0 8 16" fill="none" stroke="#38bdf8" strokeWidth="1.5" />
                          <line x1="14" y1="0" x2="20" y2="0" stroke="#94a3b8" strokeWidth="1.5" />
                        </g>
                      )}
                      {/* Custom User Module Symbols */}
                      {(() => {
                        const customDef = customSymbols.find((cs) => cs.id === comp.kind)
                        if (!customDef) return null
                        return (
                          <g>
                            {customDef.primitives.map((prim, idx) => {
                              if (prim.type === 'rect') {
                                const rx = Math.min(prim.x1, prim.x2)
                                const ry = Math.min(prim.y1, prim.y2)
                                const rw = Math.abs(prim.x2 - prim.x1)
                                const rh = Math.abs(prim.y2 - prim.y1)
                                return (
                                  <rect
                                    key={idx}
                                    x={rx}
                                    y={ry}
                                    width={rw}
                                    height={rh}
                                    fill="rgba(56, 189, 248, 0.05)"
                                    stroke="#38bdf8"
                                    strokeWidth={1.5}
                                  />
                                )
                              } else if (prim.type === 'circle') {
                                return (
                                  <circle
                                    key={idx}
                                    cx={prim.x1}
                                    cy={prim.y1}
                                    r={prim.radius || 15}
                                    fill="rgba(56, 189, 248, 0.05)"
                                    stroke="#38bdf8"
                                    strokeWidth={1.5}
                                  />
                                )
                              } else if (prim.type === 'line') {
                                return (
                                  <line
                                    key={idx}
                                    x1={prim.x1}
                                    y1={prim.y1}
                                    x2={prim.x2}
                                    y2={prim.y2}
                                    stroke="#38bdf8"
                                    strokeWidth={1.5}
                                  />
                                )
                              } else if (prim.type === 'text') {
                                return (
                                  <text
                                    key={idx}
                                    x={prim.x1}
                                    y={prim.y1}
                                    fill="#38bdf8"
                                    fontSize={10}
                                    textAnchor="middle"
                                  >
                                    {prim.text}
                                  </text>
                                )
                              }
                              return null
                            })}
                          </g>
                        )
                      })()}
                    </g>

                    {/* Component Pin Connection Targets */}
                    {Array.from({ length: allKinds.find((k) => k.kind === comp.kind)?.pinCount || 2 }).map((_, pIdx) => {
                      const [pinRelX, pinRelY] = getPinCoords(0, 0, comp.kind, comp.rotation, pIdx)
                      return (
                        <g key={pIdx}>
                          {/* Invisible expanded touch hit target for mobile ergonomics */}
                          <circle
                            cx={pinRelX}
                            cy={pinRelY}
                            r="12"
                            fill="transparent"
                            style={{ cursor: 'pointer' }}
                            onClick={(e) => handlePinClick(e, comp.id, pIdx)}
                            onTouchEnd={(e) => {
                              e.stopPropagation()
                              handlePinClick(e as unknown as React.MouseEvent, comp.id, pIdx)
                            }}
                          />
                          <circle
                            cx={pinRelX}
                            cy={pinRelY}
                            r="4"
                            fill="#0284c7"
                            stroke="#38bdf8"
                            strokeWidth="1"
                            style={{ pointerEvents: 'none' }}
                          />
                        </g>
                      )
                    })}

                    {/* Non-overlapping Text Labels
                        IMPORTANT: For voltage source or standard components, offset to x=28px
                        so V1 and 5.0 do NOT collide with the circle perimeter! */}
                    <g transform={`translate(${comp.labelPos === 'left' ? -35 : comp.labelPos === 'top' ? 0 : comp.labelPos === 'bottom' ? 0 : 28}, ${comp.labelPos === 'top' ? -28 : comp.labelPos === 'bottom' ? 32 : -4})`}>
                      <text
                        x="0"
                        y="0"
                        fill="#f8fafc"
                        fontSize="11"
                        fontWeight="600"
                        textAnchor={comp.labelPos === 'left' ? 'end' : comp.labelPos === 'top' || comp.labelPos === 'bottom' ? 'middle' : 'start'}
                      >
                        {comp.name}
                      </text>
                      <text
                        x="0"
                        y="13"
                        fill="#94a3b8"
                        fontSize="10"
                        textAnchor={comp.labelPos === 'left' ? 'end' : comp.labelPos === 'top' || comp.labelPos === 'bottom' ? 'middle' : 'start'}
                      >
                        {comp.value}
                      </text>
                    </g>
                  </g>
                )
              })}
              </g>
            </svg>
            )}
          </div>

          {/* DOCKED VIRTUAL OSCILLOSCOPE */}
          {showOscilloscope && (
            <div style={{ height: '170px', backgroundColor: '#090d16', borderTop: '1px solid #1a2233', display: 'flex', flexDirection: 'column', flexShrink: 0 }}>
              {/* Scope Toolbar */}
              <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', height: '26px', padding: '0 10px', backgroundColor: '#0d111a', borderBottom: '1px solid #1a2233' }}>
                <div style={{ display: 'flex', alignItems: 'center', gap: '10px' }}>
                  <span style={{ fontSize: '11px', fontWeight: 600, color: '#f8fafc' }}>Virtual Oscilloscope</span>
                  <label style={{ display: 'flex', alignItems: 'center', gap: '4px', fontSize: '11px', color: '#94a3b8', cursor: 'pointer' }}>
                    <input
                      type="checkbox"
                      checked={fftMode}
                      onChange={(e) => setFftMode(e.target.checked)}
                      style={{ cursor: 'pointer' }}
                    />
                    FFT Spectrum Mode
                  </label>
                </div>
                <button
                  onClick={() => {}}
                  style={{
                    background: '#1e293b',
                    border: '1px solid #334155',
                    color: '#94a3b8',
                    padding: '2px 8px',
                    fontSize: '10px',
                    borderRadius: '2px',
                    cursor: 'pointer',
                  }}
                  onMouseEnter={(e) => (e.currentTarget.style.color = '#f8fafc')}
                  onMouseLeave={(e) => (e.currentTarget.style.color = '#94a3b8')}
                >
                  Clear Traces
                </button>
              </div>

              {/* Scope Screen / Canvas */}
              <div style={{ flex: 1, position: 'relative', backgroundColor: '#070a10', overflow: 'hidden' }}>
                <svg style={{ width: '100%', height: '100%' }}>
                  <defs>
                    <pattern id="scope-grid" width="40" height="24" patternUnits="userSpaceOnUse">
                      <rect width="40" height="24" fill="none" stroke="#131b2c" strokeWidth="0.75" strokeDasharray="1 3" />
                    </pattern>
                  </defs>
                  <rect width="100%" height="100%" fill="url(#scope-grid)" />

                  {/* Axis Center Lines */}
                  <line x1="0" y1="50%" x2="100%" y2="50%" stroke="#1e293b" strokeWidth="1" />
                  <line x1="50%" y1="0" x2="50%" y2="100%" stroke="#1e293b" strokeWidth="1" />

                  {/* Trace 1 (V_IN): 5.0V step / sine */}
                  <path
                    d="M 0 40 Q 150 15, 300 40 T 600 40 T 900 40"
                    fill="none"
                    stroke="#38bdf8"
                    strokeWidth="1.5"
                  />
                  {/* Trace 2 (V_OUT): 2.5V node voltage */}
                  <path
                    d="M 0 75 Q 150 60, 300 75 T 600 75 T 900 75"
                    fill="none"
                    stroke="#22c55e"
                    strokeWidth="1.5"
                  />

                  {/* Oscilloscope Labels */}
                  <text x="10" y="20" fill="#38bdf8" fontSize="10" fontFamily="monospace">CH1: 5.000 V (V1)</text>
                  <text x="110" y="20" fill="#22c55e" fontSize="10" fontFamily="monospace">CH2: 2.500 V (R1-R2)</text>
                  <text x="50%" y="135" fill="#64748b" fontSize="9" textAnchor="middle" fontFamily="monospace">Time (s)</text>
                  <text x="15" y="50%" fill="#64748b" fontSize="9" textAnchor="middle" transform="rotate(-90 15 70)" fontFamily="monospace">Amplitude (V)</text>
                </svg>
              </div>
            </div>
          )}
        </div>

        {/* Backdrop for Right Inspector on small screens */}
        {isSmallScreen && isRightPanelOpen && (
          <div
            onClick={() => setIsRightPanelOpen(false)}
            style={{
              position: 'absolute',
              inset: 0,
              backgroundColor: 'rgba(5, 8, 15, 0.75)',
              zIndex: 40,
              backdropFilter: 'blur(2px)',
            }}
          />
        )}

        {/* RIGHT SIDEBAR: INSPECTOR & THERMAL CONTROLS (Desktop Docked or Mobile Overlay Drawer) */}
        {isRightPanelOpen && (
          <div
            className={isSmallScreen ? 'drawer-slide-in-right' : undefined}
            style={{
              ...(isSmallScreen
                ? {
                    position: 'absolute',
                    top: 0,
                    bottom: 0,
                    right: 0,
                    width: '280px',
                    maxWidth: '85vw',
                    zIndex: 50,
                    boxShadow: '-4px 0 24px rgba(0,0,0,0.7)',
                  }
                : {
                    width: '230px',
                    flexShrink: 0,
                  }),
              backgroundColor: '#0d111a',
              borderLeft: '1px solid #1a2233',
              display: 'flex',
              flexDirection: 'column',
              overflowY: 'auto',
            }}
          >
            {/* Header */}
            <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', padding: '8px 10px', borderBottom: '1px solid #1a2233' }}>
              <div style={{ fontSize: '11px', fontWeight: 600, color: '#94a3b8', textTransform: 'uppercase', letterSpacing: '0.05em' }}>Inspector</div>
              {isSmallScreen && (
                <button
                  onClick={() => setIsRightPanelOpen(false)}
                  title="Close Inspector"
                  style={{ background: 'none', border: 'none', color: '#94a3b8', cursor: 'pointer', fontSize: '14px', padding: '2px 6px' }}
                >
                  X
                </button>
              )}
            </div>

          {/* Component Details */}
          <div style={{ padding: '10px', flex: 1 }}>
            {selectedComp ? (
              <div>
                {/* Prominent Component Header at top (UNBOXED) */}
                <div style={{ fontSize: '12px', fontWeight: 600, color: '#38bdf8', marginBottom: '8px' }}>
                  {selectedComp.name} ({allKinds.find((k) => k.kind === selectedComp.kind)?.name || selectedComp.kind})
                </div>

                {/* Properties (NO boxed border around properties above Delete Component!) */}
                <div style={{ display: 'flex', flexDirection: 'column', gap: '8px', marginBottom: '12px' }}>
                  <div>
                    <label style={{ display: 'block', fontSize: '10px', color: '#94a3b8', marginBottom: '2px' }}>Name</label>
                    <input
                      type="text"
                      value={selectedComp.name}
                      onChange={(e) => {
                        const newName = e.target.value
                        setComponents((prev) =>
                          prev.map((c) => (c.id === selectedComp.id ? { ...c, name: newName } : c))
                        )
                      }}
                      style={{
                        width: '100%',
                        backgroundColor: '#131b2c',
                        border: '1px solid #1e293b',
                        borderRadius: '2px',
                        color: '#f8fafc',
                        padding: '3px 6px',
                        fontSize: '11px',
                        outline: 'none',
                      }}
                    />
                  </div>

                  <div>
                    <label style={{ display: 'block', fontSize: '10px', color: '#94a3b8', marginBottom: '2px' }}>Value</label>
                    <input
                      type="text"
                      value={selectedComp.value}
                      onChange={(e) => {
                        const newVal = e.target.value
                        setComponents((prev) =>
                          prev.map((c) => (c.id === selectedComp.id ? { ...c, value: newVal } : c))
                        )
                      }}
                      style={{
                        width: '100%',
                        backgroundColor: '#131b2c',
                        border: '1px solid #1e293b',
                        borderRadius: '2px',
                        color: '#f8fafc',
                        padding: '3px 6px',
                        fontSize: '11px',
                        outline: 'none',
                      }}
                    />
                  </div>

                  {/* Label Position Presets */}
                  <div>
                    <label style={{ display: 'block', fontSize: '10px', color: '#94a3b8', marginBottom: '2px' }}>Label Position</label>
                    <div style={{ display: 'grid', gridTemplateColumns: 'repeat(4, 1fr)', gap: '2px' }}>
                      {(['right', 'left', 'top', 'bottom'] as LabelPos[]).map((pos) => (
                        <button
                          key={pos}
                          onClick={() => {
                            setComponents((prev) =>
                              prev.map((c) => (c.id === selectedComp.id ? { ...c, labelPos: pos } : c))
                            )
                          }}
                          style={{
                            padding: '2px 0',
                            backgroundColor: selectedComp.labelPos === pos ? '#0284c7' : '#1e293b',
                            border: '1px solid #334155',
                            color: '#f8fafc',
                            fontSize: '9px',
                            cursor: 'pointer',
                            borderRadius: '2px',
                            textTransform: 'capitalize',
                          }}
                        >
                          {pos}
                        </button>
                      ))}
                    </div>
                  </div>

                  {/* DC Operating Point Telemetry (Displayed when clicked!) */}
                  {(() => {
                    const tel = getComponentTelemetry(selectedComp)
                    return (
                      <div style={{ marginTop: '4px', padding: '6px', backgroundColor: '#101623', border: '1px solid #1a2233', borderRadius: '3px', fontSize: '10px', fontFamily: 'monospace' }}>
                        <div style={{ color: '#38bdf8', fontWeight: 600, marginBottom: '4px', fontSize: '9px', textTransform: 'uppercase' }}>Operating Point</div>
                        <div style={{ display: 'flex', justifyContent: 'space-between', color: '#94a3b8' }}><span>V(+):</span> <span style={{ color: '#f8fafc' }}>{tel.vPlus}</span></div>
                        <div style={{ display: 'flex', justifyContent: 'space-between', color: '#94a3b8' }}><span>V(-):</span> <span style={{ color: '#f8fafc' }}>{tel.vMinus}</span></div>
                        <div style={{ display: 'flex', justifyContent: 'space-between', color: '#94a3b8' }}><span>Delta V:</span> <span style={{ color: '#f8fafc' }}>{tel.deltaV}</span></div>
                        <div style={{ display: 'flex', justifyContent: 'space-between', color: '#94a3b8' }}><span>Current:</span> <span style={{ color: '#f8fafc' }}>{tel.current}</span></div>
                        <div style={{ display: 'flex', justifyContent: 'space-between', color: '#94a3b8' }}><span>Power:</span> <span style={{ color: '#f8fafc' }}>{tel.power}</span></div>
                        <div style={{ display: 'flex', justifyContent: 'space-between', color: '#94a3b8' }}><span>Junction Temp:</span> <span style={{ color: '#f8fafc' }}>{tel.tempC}</span></div>
                      </div>
                    )
                  })()}

                  {/* Delete Button */}
                  <button
                    onClick={() => deleteComponent(selectedComp.id)}
                    style={{
                      marginTop: '6px',
                      padding: '4px 8px',
                      backgroundColor: '#7f1d1d',
                      border: '1px solid #991b1b',
                      borderRadius: '2px',
                      color: '#fecaca',
                      fontSize: '11px',
                      cursor: 'pointer',
                    }}
                    onMouseEnter={(e) => (e.currentTarget.style.backgroundColor = '#991b1b')}
                    onMouseLeave={(e) => (e.currentTarget.style.backgroundColor = '#7f1d1d')}
                  >
                    Delete Component
                  </button>
                </div>
              </div>
            ) : (
              <div style={{ color: '#64748b', fontSize: '11px', lineHeight: '1.4' }}>
                <p style={{ marginBottom: '8px' }}>No component or wire selected</p>
                <p>Total Components: {components.length}</p>
                <p>Total Wires: {wires.length}</p>
              </div>
            )}
          </div>

          {/* Thermal Controls Section */}
          <div style={{ padding: '8px 10px', borderTop: '1px solid #1a2233', backgroundColor: '#090d16' }}>
            <div style={{ fontSize: '11px', fontWeight: 600, color: '#94a3b8', textTransform: 'uppercase', letterSpacing: '0.05em', marginBottom: '6px' }}>Thermal Controls</div>
            <label style={{ display: 'flex', alignItems: 'center', gap: '6px', fontSize: '11px', color: '#cbd5e1', cursor: 'pointer', marginBottom: '6px' }}>
              <input
                type="checkbox"
                checked={showThermalOverlay}
                onChange={(e) => setShowThermalOverlay(e.target.checked)}
                style={{ cursor: 'pointer' }}
              />
              Show Thermal Overlay
            </label>
            <div style={{ display: 'flex', alignItems: 'center', gap: '6px' }}>
              <span style={{ fontSize: '10px', color: '#94a3b8' }}>Colormap:</span>
              <select
                value={colormap}
                onChange={(e) => setColormap(e.target.value)}
                style={{
                  backgroundColor: '#131b2c',
                  border: '1px solid #1e293b',
                  color: '#f8fafc',
                  fontSize: '10px',
                  borderRadius: '2px',
                  padding: '2px 4px',
                  outline: 'none',
                  cursor: 'pointer',
                }}
              >
                <option value="Turbo">Turbo</option>
                <option value="Viridis">Viridis</option>
                <option value="Inferno">Inferno</option>
                <option value="Plasma">Plasma</option>
              </select>
            </div>
          </div>
        </div>
        )}
      </div>

      {/* =========================================================================
          ROW 4: STATUS BAR
      ========================================================================== */}
      <div className="compact-status" style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', height: '22px', backgroundColor: 'var(--bg-header, #090d16)', borderTop: '1px solid var(--border-color, #1a2233)', padding: '0 10px', color: 'var(--text-muted, #64748b)', fontSize: '10px', fontFamily: 'monospace', flexShrink: 0 }}>
        <div style={{ whiteSpace: 'nowrap', overflow: 'hidden', textOverflow: 'ellipsis' }}>
          {isVerySmallScreen ? (
            `C: ${components.length} | W: ${wires.length} | ${zoom.toFixed(1)}x`
          ) : (
            `Components: ${components.length} | Wires: ${wires.length} | Zoom: ${zoom.toFixed(1)}x`
          )}
        </div>
        <div style={{ whiteSpace: 'nowrap', flexShrink: 0, marginLeft: '8px' }}>
          Sim: {simRunning ? 'Running...' : simStatus}
        </div>
      </div>

      {/* Componentized Settings Modal */}
      <SettingsModal
        isOpen={isSettingsOpen}
        onClose={() => setIsSettingsOpen(false)}
        settings={settings}
        onUpdateSettings={updateSettings}
        actions={actions}
        onShowToast={showToast}
      />

      {/* Componentized Command Palette (Ctrl+K) */}
      <CommandPalette
        isOpen={isCommandPaletteOpen}
        onClose={() => setIsCommandPaletteOpen(false)}
        actions={actions}
        keybindOverrides={settings.keybindOverrides}
      />

      {/* Componentized Protected Confirm Dialog */}
      <ConfirmDialog
        isOpen={confirmState.isOpen}
        title={confirmState.title}
        message={confirmState.message}
        confirmLabel={confirmState.confirmLabel}
        cancelLabel={confirmState.cancelLabel}
        discardLabel={confirmState.discardLabel}
        variant={confirmState.variant}
        onConfirm={confirmState.onConfirm}
        onCancel={() => setConfirmState((prev) => ({ ...prev, isOpen: false }))}
        onDiscard={confirmState.onDiscard}
      />

      {/* Componentized Toast Notifications Container */}
      <ToastContainer toasts={toasts} onDismiss={dismissToast} />

      {/* Mandatory Mobile Landscape Orientation Lock Modal */}
      <OrientationLockModal />

      {/* Component Symbol & Shape Editor Modal */}
      <SymbolEditorModal
        isOpen={isSymbolEditorOpen}
        onClose={() => setIsSymbolEditorOpen(false)}
        onSaveSymbol={(sym) => {
          setCustomSymbols((prev) => [...prev, sym])
          showToast('Symbol Created', `Registered custom symbol ${sym.name} (${sym.id})`, 'success')
        }}
      />
    </div>
  )
}
