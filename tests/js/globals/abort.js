// WinterTC Minimum Common API: DOM Standard, `AbortController`, `EventTarget` and `Event`.

function delay(ms) {
    return new Promise((resolve) => setTimeout(resolve, ms));
}

test('AbortController is a constructor', () => {
    assertType(AbortController, 'function');
});

test('AbortController exposes a signal', () => {
    const controller = new AbortController();

    assert(controller.signal instanceof AbortSignal, 'signal is not an AbortSignal');
    assertEqual(controller.signal.aborted, false);
});

test('AbortController abort flips the signal', () => {
    const controller = new AbortController();
    controller.abort();

    assertEqual(controller.signal.aborted, true);
});

test('AbortController abort reason defaults to AbortError', () => {
    const controller = new AbortController();
    controller.abort();

    assert(controller.signal.reason instanceof DOMException, 'reason is not a DOMException');
    assertEqual(controller.signal.reason.name, 'AbortError');
});

test('AbortController abort takes a reason', () => {
    const controller = new AbortController();
    controller.abort('because');

    assertEqual(controller.signal.reason, 'because');
});

test('AbortController abort fires an abort event', () => {
    const controller = new AbortController();
    let fired = 0;
    controller.signal.addEventListener('abort', () => { fired++; });
    controller.abort();

    assertEqual(fired, 1);
});

test('AbortController abort fires only once', () => {
    const controller = new AbortController();
    let fired = 0;
    controller.signal.addEventListener('abort', () => { fired++; });
    controller.abort();
    controller.abort();

    assertEqual(fired, 1);
});

test('AbortSignal honours the once option', () => {
    const controller = new AbortController();
    let fired = 0;
    controller.signal.addEventListener('abort', () => { fired++; }, { once: true });
    controller.signal.dispatchEvent(new Event('abort'));
    controller.signal.dispatchEvent(new Event('abort'));

    assertEqual(fired, 1);
});

test('AbortSignal supports onabort', () => {
    const controller = new AbortController();
    let fired = false;
    controller.signal.onabort = () => { fired = true; };
    controller.abort();

    assertEqual(fired, true);
});

test('AbortSignal removeEventListener drops the listener', () => {
    const controller = new AbortController();
    let fired = false;
    const listener = () => { fired = true; };
    controller.signal.addEventListener('abort', listener);
    controller.signal.removeEventListener('abort', listener);
    controller.abort();

    assertEqual(fired, false);
});

test('AbortSignal.abort returns an aborted signal', () => {
    const signal = AbortSignal.abort();

    assertEqual(signal.aborted, true);
    assertEqual(signal.reason.name, 'AbortError');
});

test('AbortSignal.abort takes a reason', () => {
    assertEqual(AbortSignal.abort('why').reason, 'why');
});

test('AbortSignal.timeout aborts after the delay', async () => {
    const signal = AbortSignal.timeout(10);

    assertEqual(signal.aborted, false);
    await delay(40);
    assertEqual(signal.aborted, true);
    assertEqual(signal.reason.name, 'TimeoutError');
});

test('AbortSignal throwIfAborted throws the reason', () => {
    const controller = new AbortController();
    controller.signal.throwIfAborted();
    controller.abort('stop');

    assertThrows(() => controller.signal.throwIfAborted(), 'stop');
});

test('AbortSignal.any aborts with the first signal', () => {
    const first = new AbortController();
    const second = new AbortController();
    const combined = AbortSignal.any([first.signal, second.signal]);

    assertEqual(combined.aborted, false);
    second.abort('second');
    assertEqual(combined.aborted, true);
    assertEqual(combined.reason, 'second');
});

test('EventTarget is a constructor', () => {
    assertType(EventTarget, 'function');
});

test('EventTarget dispatches to a listener', () => {
    const target = new EventTarget();
    let seen = null;
    target.addEventListener('ping', (event) => { seen = event.type; });
    target.dispatchEvent(new Event('ping'));

    assertEqual(seen, 'ping');
});

test('EventTarget calls listeners in order', () => {
    const target = new EventTarget();
    const order = [];
    target.addEventListener('ping', () => order.push(1));
    target.addEventListener('ping', () => order.push(2));
    target.dispatchEvent(new Event('ping'));

    assertArrayEqual(order, [1, 2]);
});

test('EventTarget ignores a duplicate listener', () => {
    const target = new EventTarget();
    let fired = 0;
    const listener = () => { fired++; };
    target.addEventListener('ping', listener);
    target.addEventListener('ping', listener);
    target.dispatchEvent(new Event('ping'));

    assertEqual(fired, 1);
});

test('Event exposes its type and defaults', () => {
    const event = new Event('ping');

    assertEqual(event.type, 'ping');
    assertEqual(event.bubbles, false);
    assertEqual(event.cancelable, false);
    assertEqual(event.defaultPrevented, false);
});

test('Event preventDefault sets defaultPrevented', () => {
    const event = new Event('ping', { cancelable: true });
    event.preventDefault();

    assertEqual(event.defaultPrevented, true);
});

test('CustomEvent carries a detail', () => {
    const event = new CustomEvent('ping', { detail: { a: 1 } });

    assertEqual(event.detail.a, 1);
});

test('DOMException is a constructor', () => {
    const error = new DOMException('boom', 'DataError');

    assertEqual(error.name, 'DataError');
    assertEqual(error.message, 'boom');
    assert(error instanceof Error, 'not an Error');
});

test('DOMException defaults to Error', () => {
    assertEqual(new DOMException('boom').name, 'Error');
});
