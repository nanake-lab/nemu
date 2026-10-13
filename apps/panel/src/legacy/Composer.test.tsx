import { expect, test } from 'vitest';
import { renderToStaticMarkup } from 'react-dom/server';
import { Composer } from './Composer';

test('legacy composer escapes text and prevents empty or disconnected submission', () => {
  const render = (draft: string, disabled: boolean) =>
    renderToStaticMarkup(
      <Composer draft={draft} setDraft={() => {}} onSubmit={() => {}} disabled={disabled} />,
    );
  expect(render('', false)).toContain('disabled=""');
  expect(render('hello', true)).toContain('disabled=""');
  expect(render('hello', false)).not.toContain('disabled=""');
  expect(render('<script>unsafe</script>', false)).toContain('&lt;script&gt;unsafe&lt;/script&gt;');
});
