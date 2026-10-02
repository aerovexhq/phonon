import React, { useState, useEffect } from 'react'

export const OrientationLockModal: React.FC = () => {
  const [isPortrait, setIsPortrait] = useState<boolean>(false)
  const [isTouch, setIsTouch] = useState<boolean>(false)

  useEffect(() => {
    const checkOrientation = () => {
      const portrait = window.innerHeight > window.innerWidth
      const touch =
        'ontouchstart' in window ||
        navigator.maxTouchPoints > 0 ||
        (window.matchMedia && window.matchMedia('(pointer: coarse)').matches)
      const isMobileWidth = Math.min(window.innerWidth, window.innerHeight) <= 900
      setIsPortrait(portrait && (touch || isMobileWidth))
      setIsTouch(touch)
    }

    checkOrientation()
    window.addEventListener('resize', checkOrientation)
    window.addEventListener('orientationchange', checkOrientation)

    return () => {
      window.removeEventListener('resize', checkOrientation)
      window.removeEventListener('orientationchange', checkOrientation)
    }
  }, [])

  const handleRequestLandscape = async () => {
    try {
      if (document.documentElement.requestFullscreen) {
        await document.documentElement.requestFullscreen()
      }
      if (window.screen.orientation && 'lock' in window.screen.orientation) {
        // @ts-ignore
        await window.screen.orientation.lock('landscape')
      }
    } catch {
      // Ignore if user gesture or browser does not support programmatic lock
    }
  }

  if (!isPortrait) return null

  return (
    <div
      style={{
        position: 'fixed',
        inset: 0,
        zIndex: 9999,
        backgroundColor: 'rgba(7, 10, 19, 0.98)',
        display: 'flex',
        flexDirection: 'column',
        alignItems: 'center',
        justifyContent: 'center',
        padding: '24px',
        textAlign: 'center',
        userSelect: 'none',
        backdropFilter: 'blur(16px)',
      }}
    >
      {/* Animated Phone Rotation SVG */}
      <div style={{ position: 'relative', width: '110px', height: '110px', marginBottom: '22px' }}>
        <svg
          viewBox="0 0 100 100"
          style={{
            width: '100%',
            height: '100%',
            animation: 'phoneRotateAnimation 2.4s ease-in-out infinite',
            transformOrigin: '50px 50px',
          }}
        >
          {/* Outer Rounded Phone Body */}
          <rect
            x="32"
            y="18"
            width="36"
            height="64"
            rx="6"
            fill="#131b2c"
            stroke="#38bdf8"
            strokeWidth="2.5"
          />
          {/* Screen Inner */}
          <rect
            x="36"
            y="26"
            width="28"
            height="48"
            rx="2"
            fill="#090d16"
            stroke="#1e293b"
            strokeWidth="1"
          />
          {/* Home Bar / Speaker */}
          <line x1="45" y1="22" x2="55" y2="22" stroke="#64748b" strokeWidth="2" strokeLinecap="round" />
          <circle cx="50" cy="78" r="2.5" fill="#38bdf8" />
        </svg>

        {/* Circular Rotation Guide Arrows */}
        <svg
          viewBox="0 0 100 100"
          style={{
            position: 'absolute',
            inset: 0,
            width: '100%',
            height: '100%',
            pointerEvents: 'none',
          }}
        >
          <path
            d="M 20 50 A 30 30 0 0 1 80 50"
            fill="none"
            stroke="#0284c7"
            strokeWidth="1.8"
            strokeDasharray="4 4"
          />
          <polygon points="80,44 86,52 78,54" fill="#38bdf8" />
        </svg>
      </div>

      {/* Badge */}
      <div
        style={{
          display: 'inline-block',
          padding: '4px 12px',
          borderRadius: '999px',
          backgroundColor: '#0c2238',
          border: '1px solid #0284c7',
          color: '#38bdf8',
          fontSize: '11px',
          fontWeight: 600,
          textTransform: 'uppercase',
          letterSpacing: '0.08em',
          marginBottom: '12px',
        }}
      >
        Landscape Orientation Required
      </div>

      {/* Heading */}
      <h2
        style={{
          fontSize: '20px',
          fontWeight: 700,
          color: '#f8fafc',
          marginBottom: '10px',
          letterSpacing: '-0.02em',
        }}
      >
        Please Rotate Your Device
      </h2>

      {/* Description */}
      <p
        style={{
          maxWidth: '380px',
          fontSize: '13px',
          lineHeight: '1.6',
          color: '#94a3b8',
          marginBottom: '24px',
        }}
      >
        Phonon Electronic CAD Studio requires a horizontal widescreen layout for precision circuit schematic routing, component placement, and oscilloscope waveform inspection.
      </p>

      {/* Interactive Action Button */}
      {isTouch && (
        <button
          onClick={handleRequestLandscape}
          style={{
            display: 'inline-flex',
            alignItems: 'center',
            gap: '8px',
            padding: '10px 20px',
            backgroundColor: '#0284c7',
            color: '#ffffff',
            border: 'none',
            borderRadius: '6px',
            fontSize: '13px',
            fontWeight: 600,
            cursor: 'pointer',
            transition: 'background-color 0.15s ease',
            boxShadow: '0 4px 12px rgba(2, 132, 199, 0.3)',
          }}
          onMouseEnter={(e) => (e.currentTarget.style.backgroundColor = '#0369a1')}
          onMouseLeave={(e) => (e.currentTarget.style.backgroundColor = '#0284c7')}
        >
          <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
            <rect x="2" y="6" width="20" height="12" rx="2" />
            <path d="M12 12h.01" />
          </svg>
          Switch to Fullscreen Landscape
        </button>
      )}

      {/* Aspect Ratio Note */}
      <div
        style={{
          marginTop: '20px',
          fontSize: '11px',
          color: '#64748b',
          fontFamily: 'monospace',
        }}
      >
        Target Aspect Ratio: 16:9 / 21:9 Horizontal
      </div>
    </div>
  )
}
