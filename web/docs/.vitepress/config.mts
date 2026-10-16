import { defineConfig } from 'vitepress'

export default defineConfig({
  title: 'Phonon',
  description: 'Universal Multi-Scale Visual CAD Studio & Semiconductor Solver',
  cleanUrls: true,
  ignoreDeadLinks: true,
  appearance: 'dark',
  head: [
    ['link', { rel: 'icon', type: 'image/svg+xml', href: '/favicon.svg' }],
    ['meta', { name: 'theme-color', content: '#08090a' }],
    ['meta', { name: 'color-scheme', content: 'dark' }],
    ['meta', { property: 'og:type', content: 'website' }],
    ['meta', { property: 'og:title', content: 'Phonon — Universal Multi-Scale Visual CAD Studio' }],
    ['meta', { property: 'og:description', content: 'Industry-grade electro-thermal circuit simulator, cryogenic CMOS modeler, and visual CAD studio.' }],
    ['link', { rel: 'preconnect', href: 'https://fonts.googleapis.com' }],
    ['link', { rel: 'preconnect', href: 'https://fonts.gstatic.com', crossorigin: '' }],
    ['link', { href: 'https://fonts.googleapis.com/css2?family=Inter:wght@300;400;500;600;700;800&family=JetBrains+Mono:wght@400;500;600;700&display=swap', rel: 'stylesheet' }]
  ],
  themeConfig: {
    logo: '/favicon.svg',
    siteTitle: 'Phonon.',
    search: {
      provider: 'local'
    },
    nav: [
      { text: 'Guide', link: '/guide/getting-started' },
      { text: 'Installation', link: '/guide/installation' },
      { text: '6 Realism Tiers', link: '/tiers/overview' },
      { text: 'CLI Reference', link: '/cli/commands' },
      { text: 'Web Studio', link: '/studio/', target: '_self' },
      {
        text: 'Aerovex Ecosystem',
        items: [
          { text: 'Aerovex Platform', link: 'https://aerovex.net', target: '_blank' },
          { text: 'Axiom EDA', link: 'https://axiom.aerovex.net', target: '_blank' },
          { text: 'Phonon CAD', link: 'https://phonon.aerovex.net', target: '_self' },
          { text: 'Atoma CAD', link: 'https://atoma.aerovex.net', target: '_blank' },
          { text: 'Sonon DSP', link: 'https://sonon.aerovex.net', target: '_blank' },
          { text: 'Vexel Lang', link: 'https://vexel.aerovex.net', target: '_blank' },
          { text: 'Appify Bundler', link: 'https://appify.aerovex.net', target: '_blank' }
        ]
      },
      { text: 'v0.1.0', link: 'https://github.com/aerovexhq/phonon/releases/tag/v0.1.0' }
    ],
    sidebar: [
      {
        text: 'Getting Started <span class="nx-sidebar-badge">3</span>',
        collapsed: false,
        items: [
          { text: 'Introduction & Overview', link: '/guide/getting-started' },
          { text: 'Installation & Setup', link: '/guide/installation' },
          { text: 'Desktop Visual CAD Studio', link: '/guide/desktop-studio' }
        ]
      },
      {
        text: 'Multi-Scale Realism Tiers <span class="nx-sidebar-badge">8</span>',
        collapsed: false,
        items: [
          { text: 'Architecture Overview', link: '/tiers/overview' },
          { text: 'Tier 0: Quantum Acoustics', link: '/tiers/tier-0-quantum-acoustics' },
          { text: 'Tier 1: Microscopic TCAD', link: '/tiers/tier-1-tcad' },
          { text: 'Tier 2: Inverse Multi-Objective', link: '/tiers/tier-2-inverse-design' },
          { text: 'Tier 3: SPICE Circuit Electronics', link: '/tiers/tier-3-spice' },
          { text: 'Tier 4: Cryogenic Cryo-CMOS (4.2K)', link: '/tiers/tier-4-cryo-cmos' },
          { text: 'Tier 5: Coupled Electro-Thermal MNA', link: '/tiers/tier-5-electro-thermal' },
          { text: 'Tier 6: High-Throughput SIMD & Rayon', link: '/tiers/tier-6-simd' }
        ]
      },
      {
        text: 'CLI Reference <span class="nx-sidebar-badge">6</span>',
        collapsed: false,
        items: [
          { text: 'Command Line Overview', link: '/cli/commands' },
          { text: 'Unified Executable (phonon & ui)', link: '/cli/unified-binary' },
          { text: 'Circuit Validation (ERC)', link: '/cli/validate' },
          { text: 'Transient & DC Simulation', link: '/cli/run' },
          { text: 'Parametric Sweep', link: '/cli/sweep' },
          { text: 'Monte Carlo Analysis', link: '/cli/monte-carlo' }
        ]
      }
    ],
    socialLinks: [
      { icon: 'github', link: 'https://github.com/aerovexhq/phonon' }
    ],
    footer: {
      message: 'Released under the MIT License.',
      copyright: 'Copyright (c) 2026 Aerovex Corporation. All rights reserved.'
    }
  }
})
