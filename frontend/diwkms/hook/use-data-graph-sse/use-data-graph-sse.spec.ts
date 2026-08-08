import { describe, expect, it, vi, afterEach } from 'vitest';
import { reportError } from './use-data-graph-sse.js';

describe('SPEC-018 hooks/use-data-graph-sse', () => {
  afterEach(() => {
    vi.restoreAllMocks();
  });

  // AC-4: structured error output, never bare console.log
  it('dispatches a structured app:error event and logs JSON', () => {
    const consoleSpy = vi.spyOn(console, 'error').mockImplementation(() => {});
    const listener = vi.fn();
    window.addEventListener('app:error', listener);

    reportError('test-context', new Error('kaboom'));

    expect(listener).toHaveBeenCalledTimes(1);
    const detail = (listener.mock.calls[0][0] as CustomEvent).detail;
    expect(detail).toMatchObject({ context: 'test-context', message: 'kaboom' });
    expect(detail.timestamp).toBeTruthy();

    const logged = JSON.parse(consoleSpy.mock.calls[0][0] as string);
    expect(logged).toMatchObject({ '@type': 'app:Error', context: 'test-context' });
  });
});
