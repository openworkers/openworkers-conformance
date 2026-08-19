// WinterTC Minimum Common API: HTML Standard globals, console and High Resolution Time.

test('globalThis is an object', () => {
    assertType(globalThis, 'object');
});

test('self points at the global', () => {
    assert(self === globalThis, 'self is not globalThis');
});

test('console exposes the usual levels', () => {
    assertType(console.log, 'function');
    assertType(console.info, 'function');
    assertType(console.warn, 'function');
    assertType(console.error, 'function');
    assertType(console.debug, 'function');
});

test('console.log accepts several arguments', () => {
    console.log('conformance', 1, { a: 1 }, [1, 2], null);
});

test('performance is an object', () => {
    assertType(performance, 'object');
});

test('performance.now returns a number', () => {
    assertType(performance.now(), 'number');
});

test('performance.now does not go backwards', async () => {
    const first = performance.now();
    await new Promise((resolve) => setTimeout(resolve, 5));

    assert(performance.now() >= first, 'the clock went backwards');
});

test('performance.timeOrigin is a number', () => {
    assertType(performance.timeOrigin, 'number');
});

test('navigator.userAgent is a string', () => {
    assertType(navigator.userAgent, 'string');
    assert(navigator.userAgent.length > 0, 'the user agent is empty');
});

test('reportError is a function', () => {
    assertType(reportError, 'function');
});

test('WebAssembly validates a minimal module', () => {
    const header = new Uint8Array([0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00]);

    assertEqual(WebAssembly.validate(header), true);
});

test('WebAssembly compiles a minimal module', async () => {
    const header = new Uint8Array([0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00]);
    const module = await WebAssembly.compile(header);

    assert(module instanceof WebAssembly.Module, 'not a WebAssembly.Module');
});

test('URLPattern matches a path', () => {
    const pattern = new URLPattern({ pathname: '/books/:id' });
    const match = pattern.exec('http://example.com/books/7');

    assertEqual(match.pathname.groups.id, '7');
});

test('MessageChannel delivers a message', async () => {
    const channel = new MessageChannel();
    const received = new Promise((resolve) => {
        channel.port2.onmessage = (event) => resolve(event.data);
    });
    channel.port1.postMessage('ping');
    channel.port2.start();

    assertEqual(await received, 'ping');
});

test('Intl is available', () => {
    assertType(Intl, 'object');
    assertEqual(new Intl.NumberFormat('en-US').format(1234.5), '1,234.5');
});

test('Date.toLocaleDateString honours a locale', () => {
    assertEqual(new Date(Date.UTC(2020, 0, 2)).toLocaleDateString('en-US', { timeZone: 'UTC' }), '1/2/2020');
});
