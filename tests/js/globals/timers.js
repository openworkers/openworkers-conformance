// WinterTC Minimum Common API: HTML Standard timers, `setTimeout` and `setInterval`.

function delay(ms) {
    return new Promise((resolve) => setTimeout(resolve, ms));
}

test('setTimeout is a function', () => {
    assertType(setTimeout, 'function');
});

test('clearTimeout is a function', () => {
    assertType(clearTimeout, 'function');
});

test('setInterval is a function', () => {
    assertType(setInterval, 'function');
});

test('clearInterval is a function', () => {
    assertType(clearInterval, 'function');
});

test('setTimeout runs the callback', async () => {
    let ran = false;
    setTimeout(() => { ran = true; }, 0);
    await delay(10);

    assertEqual(ran, true);
});

test('setTimeout runs after the current turn', () => {
    let ran = false;
    setTimeout(() => { ran = true; }, 0);

    assertEqual(ran, false);
});

test('setTimeout returns a handle', () => {
    const handle = setTimeout(() => {}, 1000);
    clearTimeout(handle);

    assert(handle !== undefined && handle !== null, 'no handle returned');
});

test('setTimeout returns a fresh handle each call', () => {
    const first = setTimeout(() => {}, 1000);
    const second = setTimeout(() => {}, 1000);
    clearTimeout(first);
    clearTimeout(second);

    assert(first !== second, 'the same handle came back twice');
});

test('setTimeout forwards extra arguments', async () => {
    let seen = null;
    setTimeout((a, b) => { seen = a + b; }, 0, 'x', 'y');
    await delay(10);

    assertEqual(seen, 'xy');
});

test('setTimeout treats a missing delay as zero', async () => {
    let ran = false;
    setTimeout(() => { ran = true; });
    await delay(10);

    assertEqual(ran, true);
});

test('setTimeout orders by delay', async () => {
    const order = [];
    setTimeout(() => order.push('late'), 20);
    setTimeout(() => order.push('early'), 0);
    await delay(50);

    assertArrayEqual(order, ['early', 'late']);
});

test('clearTimeout cancels the callback', async () => {
    let ran = false;
    clearTimeout(setTimeout(() => { ran = true; }, 0));
    await delay(10);

    assertEqual(ran, false);
});

test('clearTimeout of an unknown handle is a no-op', () => {
    clearTimeout(999999);
});

test('setTimeout nests', async () => {
    const order = [];
    await new Promise((resolve) => {
        setTimeout(() => {
            order.push('outer');
            setTimeout(() => {
                order.push('inner');
                resolve();
            }, 0);
        }, 0);
    });

    assertArrayEqual(order, ['outer', 'inner']);
});

test('setInterval repeats until cleared', async () => {
    let count = 0;
    const handle = setInterval(() => { count++; }, 5);
    await delay(60);
    clearInterval(handle);

    assert(count >= 2, 'fired ' + count + ' times, expected at least 2');
});

test('clearInterval stops the callback', async () => {
    let count = 0;
    const handle = setInterval(() => { count++; }, 5);
    await delay(30);
    clearInterval(handle);
    const settled = count;
    await delay(30);

    assertEqual(count, settled);
});
