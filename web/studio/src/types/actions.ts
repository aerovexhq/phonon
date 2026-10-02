// Action and Keybinding Registry Types for Phonon Studio

export type ActionCategory = 'File' | 'Edit' | 'View' | 'Simulation' | 'Tools' | 'Settings' | 'Help'

export interface ActionDefinition {
  id: string
  label: string
  category: ActionCategory
  description: string
  defaultKeybind?: string
  currentKeybind?: string
  run: () => void
}

export interface CustomTheme {
  id: string
  name: string
  variables: Record<string, string>
}

export type FontSizeOption = 'sm' | 'md' | 'lg' | 'xl'

export interface StudioSettings {
  theme: string
  fontSize: FontSizeOption
  customThemes: CustomTheme[]
  keybindOverrides: Record<string, string> // actionId -> key combination string
  gridSnap: boolean
  showGrid: boolean
  showOscilloscope: boolean
  showThermalOverlay: boolean
  maxIterations: number
  tolerance: number
}

export interface ToastMessage {
  id: string
  type: 'success' | 'info' | 'warn' | 'error'
  title: string
  message?: string
  timestamp: number
}
