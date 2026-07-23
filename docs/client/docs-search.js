export default function mount(root) {
  const input = root.querySelector('input');
  const sidebar = root.closest('aside');
  if (!sidebar) return;
  const links = [...sidebar.querySelectorAll('[data-doc-title]')];
  const status = root.querySelector('[aria-live="polite"]');
  if (!input || !status) return;
  input.addEventListener('input', () => {
    const query = input.value.trim().toLowerCase();
    let visible = 0;
    for (const link of links) {
      const matches = !query || link.dataset.docTitle.toLowerCase().includes(query);
      link.hidden = !matches;
      if (matches) visible += 1;
    }
    status.textContent = query ? `${visible} guide${visible === 1 ? '' : 's'} found` : '';
  });
}
