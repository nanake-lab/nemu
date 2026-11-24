import { createRoot } from 'react-dom/client';
import { Chat } from './chat/Chat.js';
import { DebugPanel } from './debug/index.js';
import './style.css';

const root = document.getElementById('root');
if (!root) throw new Error('Missing panel root');
createRoot(root).render(
  <main>
    <h1>Nemu</h1>
    <Chat />
    <DebugPanel />
  </main>,
);
