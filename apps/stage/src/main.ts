import './style.css';
const root = document.querySelector<HTMLDivElement>('#root');
if (!root) throw new Error('Missing stage root');
root.innerHTML =
  '<div class="placeholder" role="img" aria-label="Stage placeholder">Stage placeholder</div>';
const background = new URLSearchParams(location.search).get('bg');
if (background === 'white' || background === 'black') document.body.style.background = background;
