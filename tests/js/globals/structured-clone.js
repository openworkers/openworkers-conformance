// WinterTC Minimum Common API: HTML Standard, `structuredClone`.

test('structuredClone is a function', () => {
    assertType(structuredClone, 'function');
});

test('structuredClone copies a primitive', () => {
    assertEqual(structuredClone(42), 42);
    assertEqual(structuredClone('x'), 'x');
    assertEqual(structuredClone(null), null);
    assertEqual(structuredClone(undefined), undefined);
});

test('structuredClone copies a plain object', () => {
    const source = { a: 1, b: 'two' };
    const copy = structuredClone(source);

    assert(copy !== source, 'the same object came back');
    assertEqual(copy.a, 1);
    assertEqual(copy.b, 'two');
});

test('structuredClone copies nested values', () => {
    const source = { a: { b: [1, 2, 3] } };
    const copy = structuredClone(source);
    copy.a.b[0] = 9;

    assertEqual(source.a.b[0], 1);
    assertEqual(copy.a.b[2], 3);
});

test('structuredClone copies an array', () => {
    const copy = structuredClone([1, 'two', true]);

    assert(Array.isArray(copy), 'not an array');
    assertArrayEqual(copy, [1, 'two', true]);
});

test('structuredClone copies a Date', () => {
    const source = new Date(1700000000000);
    const copy = structuredClone(source);

    assert(copy instanceof Date, 'not a Date');
    assertEqual(copy.getTime(), source.getTime());
});

test('structuredClone copies a Map', () => {
    const copy = structuredClone(new Map([['a', 1]]));

    assert(copy instanceof Map, 'not a Map');
    assertEqual(copy.get('a'), 1);
});

test('structuredClone copies a Set', () => {
    const copy = structuredClone(new Set([1, 2]));

    assert(copy instanceof Set, 'not a Set');
    assertEqual(copy.has(2), true);
});

test('structuredClone copies a RegExp', () => {
    const copy = structuredClone(/ab+c/gi);

    assert(copy instanceof RegExp, 'not a RegExp');
    assertEqual(copy.source, 'ab+c');
    assertEqual(copy.flags, 'gi');
});

test('structuredClone copies an ArrayBuffer', () => {
    const source = new Uint8Array([1, 2, 3]).buffer;
    const copy = structuredClone(source);

    assert(copy !== source, 'the same buffer came back');
    assertArrayEqual(new Uint8Array(copy), [1, 2, 3]);
});

test('structuredClone copies a typed array', () => {
    const copy = structuredClone(new Uint16Array([1, 2]));

    assert(copy instanceof Uint16Array, 'not a Uint16Array');
    assertArrayEqual(copy, [1, 2]);
});

test('structuredClone copies an Error', () => {
    const copy = structuredClone(new TypeError('boom'));

    assert(copy instanceof Error, 'not an Error');
    assertEqual(copy.name, 'TypeError');
    assertEqual(copy.message, 'boom');
});

test('structuredClone keeps a cycle', () => {
    const source = { name: 'root' };
    source.self = source;
    const copy = structuredClone(source);

    assert(copy.self === copy, 'the cycle was not preserved');
});

test('structuredClone keeps a shared reference shared', () => {
    const shared = { a: 1 };
    const copy = structuredClone({ x: shared, y: shared });

    assert(copy.x === copy.y, 'the reference was duplicated');
});

test('structuredClone rejects a function', () => {
    assertThrows(() => structuredClone(() => {}), 'DataCloneError');
});

test('structuredClone rejects a symbol', () => {
    assertThrows(() => structuredClone(Symbol('x')), 'DataCloneError');
});

test('structuredClone transfers an ArrayBuffer', () => {
    const source = new Uint8Array([1, 2, 3]).buffer;
    const copy = structuredClone(source, { transfer: [source] });

    assertEqual(copy.byteLength, 3);
    assertEqual(source.byteLength, 0);
});
