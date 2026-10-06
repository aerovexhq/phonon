// Phonon Studio Progressive Web App (PWA) Offline ServiceWorker
// Provides instant offline simulation, CacheStorage precaching, and asset lifecycle management.

const CACHE_NAME = 'phonon-studio-v88f1a5d';

const PRECACHE_ASSETS = [
  '/studio/',
  '/studio/index.html',
  '/studio/favicon.svg',
  '/studio/manifest.webmanifest',
  '/studio/wasm/phonon_gui.js',
  '/studio/wasm/phonon_gui_bg.wasm',
  '/studio/wasm/build_meta.json'
];

// Install Event: Precache essential app shell and WebAssembly runtime
self.addEventListener('install', (event) => {
  self.skipWaiting();
  event.waitUntil(
    caches.open(CACHE_NAME).then((cache) => {
      return cache.addAll(PRECACHE_ASSETS).catch((err) => {
        console.warn('Precache partial fallback (some assets may be loaded on-demand):', err);
      });
    })
  );
});

// Activate Event: Purge outdated caches and claim existing clients
self.addEventListener('activate', (event) => {
  event.waitUntil(
    Promise.all([
      self.clients.claim(),
      caches.keys().then((cacheNames) => {
        return Promise.all(
          cacheNames.map((name) => {
            if (name.startsWith('phonon-studio-') && name !== CACHE_NAME) {
              console.log('Purging legacy PWA cache:', name);
              return caches.delete(name);
            }
          })
        );
      })
    ])
  );
});

// Fetch Event: Caching strategies tailored for WebAssembly binary workloads
self.addEventListener('fetch', (event) => {
  const url = new URL(event.request.url);

  // Skip non-GET requests and external origins
  if (event.request.method !== 'GET' || url.origin !== self.location.origin) {
    return;
  }

  // Strategy 1: Cache-First for WebAssembly binary and JS bindings
  if (url.pathname.includes('/wasm/')) {
    event.respondWith(
      caches.match(event.request).then((cachedResponse) => {
        if (cachedResponse) {
          return cachedResponse;
        }
        return fetch(event.request).then((networkResponse) => {
          if (networkResponse && networkResponse.status === 200) {
            const responseToCache = networkResponse.clone();
            caches.open(CACHE_NAME).then((cache) => {
              cache.put(event.request, responseToCache);
            });
          }
          return networkResponse;
        });
      })
    );
    return;
  }

  // Strategy 2: Network-First with Cache fallback for HTML navigations
  if (event.request.mode === 'navigate') {
    event.respondWith(
      fetch(event.request)
        .then((networkResponse) => {
          if (networkResponse && networkResponse.status === 200) {
            const responseToCache = networkResponse.clone();
            caches.open(CACHE_NAME).then((cache) => {
              cache.put(event.request, responseToCache);
            });
          }
          return networkResponse;
        })
        .catch(async () => {
          const cached = await caches.match(event.request);
          if (cached) return cached;
          const fallback = await caches.match('/studio/index.html');
          return fallback || new Response('Phonon Studio is offline. Please connect to internet to initial load.', {
            status: 503,
            headers: { 'Content-Type': 'text/plain' }
          });
        })
    );
    return;
  }

  // Strategy 3: Stale-While-Revalidate for static UI assets (favicon, css, manifest)
  event.respondWith(
    caches.match(event.request).then((cachedResponse) => {
      const fetchPromise = fetch(event.request)
        .then((networkResponse) => {
          if (networkResponse && networkResponse.status === 200) {
            const responseToCache = networkResponse.clone();
            caches.open(CACHE_NAME).then((cache) => {
              cache.put(event.request, responseToCache);
            });
          }
          return networkResponse;
        })
        .catch(() => cachedResponse);

      return cachedResponse || fetchPromise;
    })
  );
});
