// ARKIVE Service Worker - PWA Offline Support
const CACHE_NAME = 'arkive-v1.0.0';
const DYNAMIC_CACHE = 'arkive-dynamic-v1';

// Recursos essenciais para cache offline
const urlsToCache = [
  '/',
  '/index.html',
  '/manifest.json',
  '/assets/index.css',
  '/assets/index.js'
];

// Instalação do Service Worker
self.addEventListener('install', (event) => {
  console.log('📦 ARKIVE PWA: Instalando Service Worker...');
  
  event.waitUntil(
    caches.open(CACHE_NAME)
      .then((cache) => {
        console.log('✅ Cache aberto');
        return cache.addAll(urlsToCache);
      })
      .then(() => self.skipWaiting())
  );
});

// Ativação e limpeza de caches antigos
self.addEventListener('activate', (event) => {
  console.log('🚀 ARKIVE PWA: Service Worker ativado');
  
  const cacheWhitelist = [CACHE_NAME, DYNAMIC_CACHE];
  
  event.waitUntil(
    caches.keys().then((cacheNames) => {
      return Promise.all(
        cacheNames.map((cacheName) => {
          if (!cacheWhitelist.includes(cacheName)) {
            console.log('🗑️ Removendo cache antigo:', cacheName);
            return caches.delete(cacheName);
          }
        })
      );
    }).then(() => self.clients.claim())
  );
});

// Estratégia de cache: Network First com fallback para cache
self.addEventListener('fetch', (event) => {
  const { request } = event;
  const url = new URL(request.url);
  
  // Skip para requisições não-GET
  if (request.method !== 'GET') {
    return;
  }
  
  // API calls - sempre tentar network primeiro
  if (url.pathname.startsWith('/api/') || url.pathname.startsWith('/invoke/')) {
    event.respondWith(
      fetch(request)
        .then((response) => {
          // Clone a resposta antes de armazenar
          const responseToCache = response.clone();
          
          caches.open(DYNAMIC_CACHE).then((cache) => {
            cache.put(request, responseToCache);
          });
          
          return response;
        })
        .catch(() => {
          // Se offline, buscar do cache
          return caches.match(request);
        })
    );
    return;
  }
  
  // Assets estáticos - cache first
  event.respondWith(
    caches.match(request).then((response) => {
      if (response) {
        return response;
      }
      
      return fetch(request).then((response) => {
        // Não cachear respostas não-ok
        if (!response || response.status !== 200 || response.type !== 'basic') {
          return response;
        }
        
        const responseToCache = response.clone();
        
        caches.open(DYNAMIC_CACHE).then((cache) => {
          cache.put(request, responseToCache);
        });
        
        return response;
      });
    })
  );
});

// Sincronização em background
self.addEventListener('sync', (event) => {
  console.log('🔄 ARKIVE PWA: Sincronizando dados...');
  
  if (event.tag === 'sync-documents') {
    event.waitUntil(syncDocuments());
  }
});

// Função para sincronizar documentos pendentes
async function syncDocuments() {
  try {
    // Obter documentos pendentes do IndexedDB
    const pendingDocs = await getPendingDocuments();
    
    for (const doc of pendingDocs) {
      try {
        await fetch('/api/documents', {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify(doc)
        });
        
        // Marcar como sincronizado
        await markDocumentSynced(doc.id);
      } catch (error) {
        console.error('Erro ao sincronizar documento:', doc.id, error);
      }
    }
    
    console.log('✅ Sincronização concluída');
  } catch (error) {
    console.error('❌ Erro na sincronização:', error);
  }
}

// Helpers para IndexedDB (simplificado)
async function getPendingDocuments() {
  // Implementar acesso ao IndexedDB
  return [];
}

async function markDocumentSynced(docId) {
  // Implementar marcação no IndexedDB
  return true;
}

// Notificações push
self.addEventListener('push', (event) => {
  const options = {
    body: event.data ? event.data.text() : 'Nova atualização disponível',
    icon: '/icon-192.png',
    badge: '/icon-192.png',
    vibrate: [200, 100, 200],
    data: {
      dateOfArrival: Date.now(),
      primaryKey: 1
    },
    actions: [
      {
        action: 'explore',
        title: 'Abrir ARKIVE'
      },
      {
        action: 'close',
        title: 'Fechar'
      }
    ]
  };
  
  event.waitUntil(
    self.registration.showNotification('ARKIVE Desktop', options)
  );
});

console.log('✅ ARKIVE Service Worker carregado com sucesso');