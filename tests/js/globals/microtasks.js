// WinterTC Minimum Common API: HTML Standard, `queueMicrotask` and job ordering.

function delay(ms) {
    return new Promise((resolve) => setTimeout(resolve, ms));
}

test('queueMicrotask is a function', () => {
    assertType(queueMicrotask, 'function');
});

test('queueMicrotask defers the callback', () => {
    let ran = false;
    queueMicrotask(() => { ran = true; });

    assertEqual(ran, false);
});

test('queueMicrotask runs before the next await resumes', async () => {
    let ran = false;
    queueMicrotask(() => { ran = true; });
    await null;

    assertEqual(ran, true);
});

test('queueMicrotask keeps insertion order', async () => {
    const order = [];
    queueMicrotask(() => order.push(1));
    queueMicrotask(() => order.push(2));
    queueMicrotask(() => order.push(3));
    await null;

    assertArrayEqual(order, [1, 2, 3]);
});

test('queueMicrotask runs before a zero timer', async () => {
    const order = [];
    setTimeout(() => order.push('timer'), 0);
    queueMicrotask(() => order.push('micro'));
    await delay(20);

    assertArrayEqual(order, ['micro', 'timer']);
});

test('a promise job runs before a zero timer', async () => {
    const order = [];
    setTimeout(() => order.push('timer'), 0);
    Promise.resolve().then(() => order.push('promise'));
    await delay(20);

    assertArrayEqual(order, ['promise', 'timer']);
});

test('queueMicrotask and promise jobs share one queue', async () => {
    const order = [];
    Promise.resolve().then(() => order.push('promise'));
    queueMicrotask(() => order.push('micro'));
    await delay(20);

    assertArrayEqual(order, ['promise', 'micro']);
});

test('a microtask queued from a microtask runs in the same drain', async () => {
    const order = [];
    setTimeout(() => order.push('timer'), 0);
    queueMicrotask(() => {
        order.push('outer');
        queueMicrotask(() => order.push('inner'));
    });
    await delay(20);

    assertArrayEqual(order, ['outer', 'inner', 'timer']);
});

test('queueMicrotask rejects a non-callable argument', () => {
    assertThrows(() => queueMicrotask(1), 'TypeError');
});

test('await resumes before a zero timer', async () => {
    const order = [];
    setTimeout(() => order.push('timer'), 0);
    await Promise.resolve().then(() => order.push('await'));
    order.push('after');

    assertArrayEqual(order, ['await', 'after']);
});
