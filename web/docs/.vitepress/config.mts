import { defineConfig } from 'vitepress'

export default defineConfig({
  title: 'Phonon',
  description: 'Universal Multi-Scale Visual CAD Studio & Semiconductor Solver',
  cleanUrls: true,
  ignoreDeadLinks: true,
  head: [
    ['link', { rel: 'icon', type: 'image/svg+xml', href: '/favicon.svg' }],
    ['meta', { name: 'theme-color', content: '#0f172a' }],
    ['meta', { property: 'og:type', content: 'website' }],
    ['meta', { property: 'og:title', content: 'Phonon — Universal Multi-Scale Visual CAD Studio' }],
    ['meta', { property: 'og:description', content: 'Industry-grade electro-thermal circuit simulator, cryogenic CMOS modeler, and visual CAD studio.' }]
  ],
  themeConfig: {
    logo: '/favicon.svg',
    siteTitle: 'Phonon',
    nav: [
      { text: 'Guide', link: '/guide/getting-started' },
      { text: 'Installation', link: '/guide/installation' },
      { text: '6 Realism Tiers', link: '/tiers/overview' },
      { text: 'CLI Reference', link: '/cli/commands' },
      { text: 'Web Studio', link: '/studio/', target: '_self' },
      { text: 'v0.1.0', link: 'https://github.com/aerovexsim/phonon/releases/tag/v0.1.0' }
    ],
    sidebar: {
      '/guide/': [
        {
          text: 'Getting Started',
          items: [
            { text: 'Introduction & Overview', link: '/guide/getting-started' },
            { text: 'Installation & Setup', link: '/guide/installation' },
            { text: 'Desktop Visual CAD Studio', link: '/guide/desktop-studio' }
          ]
        }
      ],
      '/tiers/': [
        {
          text: 'Multi-Scale Realism Tiers',
          items: [
            { text: 'Architecture Overview', link: '/tiers/overview' },
            { text: 'Tier 0: Topological Quantum Acoustics', link: '/tiers/tier-0-quantum-acoustics' },
            { text: 'Tier 1: Microscopic TCAD', link: '/tiers/tier-1-tcad' },
            { text: 'Tier 2: Inverse Multi-Objective Synthesis', link: '/tiers/tier-2-inverse-design' },
            { text: 'Tier 3: SPICE Circuit Electronics', link: '/tiers/tier-3-spice' },
            { text: 'Tier 4: Cryogenic Cryo-CMOS (4.2K)', link: '/tiers/tier-4-cryo-cmos' },
            { text: 'Tier 5: Coupled Electro-Thermal MNA', link: '/tiers/tier-5-electro-thermal' },
            { text: 'Tier 6: High-Throughput SIMD & Rayon', link: '/tiers/tier-6-simd' }
          ]
        }
      ],
      '/cli/': [
        {
          text: 'CLI Reference',
          items: [
            { text: 'Command Line Overview', link: '/cli/commands' },
            { text: 'Unified Executable (phonon & phonon ui)', link: '/cli/unified-binary' },
            { text: 'Circuit Validation (ERC)', link: '/cli/validate' },
            { text: 'Transient & DC Simulation', link: '/cli/run' },
            { text: 'Parametric Sweep', link: '/cli/sweep' },
            { text: 'Monte Carlo Analysis', link: '/cli/monte-carlo' }
          ]
        }
      ]
    },
    socialLinks: [
      { icon: 'github', link: 'https://github.com/aerovexsim/phonon' }
    ],
    footer: {
      message: 'Released under the MIT License.',
      copyright: 'Copyright (c) 2026 Aerovex Engineers. All rights reserved.'
    }
  }
})
