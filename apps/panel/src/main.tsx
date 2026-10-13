import { createRoot } from 'react-dom/client';
import { useState } from 'react';
import { Composer } from './legacy/Composer';
import './style.css';
const root = document.getElementById('root');
if (!root) throw new Error('Missing panel root');
function Panel() {
  const [draft, setDraft] = useState('');
  return (
    <main>
      <h1>Nemu</h1>
      <p>Application scaffold</p>
      <p>Core, conversation, and voice are not connected.</p>
      <Composer draft={draft} setDraft={setDraft} onSubmit={() => {}} disabled />
    </main>
  );
}
createRoot(root).render(<Panel />);
