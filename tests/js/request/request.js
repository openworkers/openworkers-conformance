// WinterTC Minimum Common API: Fetch Standard, the `Request` interface.

test('Request is a constructor', () => {
    assertType(Request, 'function');
});

test('Request defaults to GET', () => {
    const request = new Request('http://example.com/a');

    assertEqual(request.url, 'http://example.com/a');
    assertEqual(request.method, 'GET');
});

test('Request uppercases a known method', () => {
    assertEqual(new Request('http://example.com/', { method: 'post' }).method, 'POST');
});

test('Request rejects a relative url', () => {
    assertThrows(() => new Request('/a'), 'TypeError');
});

test('Request exposes Headers', () => {
    const request = new Request('http://example.com/', { headers: { 'X-A': '1' } });

    assert(request.headers instanceof Headers, 'headers is not a Headers');
    assertEqual(request.headers.get('x-a'), '1');
});

test('Request reads a string body as text', async () => {
    const request = new Request('http://example.com/', { method: 'POST', body: 'hello' });

    assertEqual(await request.text(), 'hello');
});

test('Request reads a body as json', async () => {
    const request = new Request('http://example.com/', { method: 'POST', body: '{"a":1}' });
    const parsed = await request.json();

    assertEqual(parsed.a, 1);
});

test('Request rejects invalid json', async () => {
    const request = new Request('http://example.com/', { method: 'POST', body: 'nope' });

    await assertRejects(request.json());
});

test('Request reads a body as an ArrayBuffer', async () => {
    const request = new Request('http://example.com/', { method: 'POST', body: 'abc' });
    const buffer = await request.arrayBuffer();

    assertEqual(buffer.byteLength, 3);
    assertEqual(new Uint8Array(buffer)[0], 97);
});

test('Request reads a typed array body', async () => {
    const body = new Uint8Array([104, 105]);
    const request = new Request('http://example.com/', { method: 'POST', body: body });

    assertEqual(await request.text(), 'hi');
});

test('Request infers a text content type', () => {
    const request = new Request('http://example.com/', { method: 'POST', body: 'x' });

    assertEqual(request.headers.get('content-type'), 'text/plain;charset=UTF-8');
});

test('Request infers a urlencoded content type', () => {
    const body = new URLSearchParams('a=1');
    const request = new Request('http://example.com/', { method: 'POST', body: body });

    assertEqual(request.headers.get('content-type'), 'application/x-www-form-urlencoded;charset=UTF-8');
});

test('Request keeps an explicit content type', () => {
    const request = new Request('http://example.com/', {
        method: 'POST',
        body: '{}',
        headers: { 'content-type': 'application/json' }
    });

    assertEqual(request.headers.get('content-type'), 'application/json');
});

test('Request tracks bodyUsed', async () => {
    const request = new Request('http://example.com/', { method: 'POST', body: 'x' });

    assertEqual(request.bodyUsed, false);
    await request.text();
    assertEqual(request.bodyUsed, true);
});

test('Request refuses a second read', async () => {
    const request = new Request('http://example.com/', { method: 'POST', body: 'x' });
    await request.text();

    await assertRejects(request.text(), 'TypeError');
});

test('Request clone reads the body twice', async () => {
    const request = new Request('http://example.com/', { method: 'POST', body: 'x' });
    const copy = request.clone();

    assertEqual(await request.text(), 'x');
    assertEqual(await copy.text(), 'x');
});

test('Request copies another Request', async () => {
    const source = new Request('http://example.com/a', { method: 'POST', body: 'x' });
    const copy = new Request(source);

    assertEqual(copy.url, 'http://example.com/a');
    assertEqual(copy.method, 'POST');
    assertEqual(await copy.text(), 'x');
});

test('Request overrides the method of a copied Request', () => {
    const source = new Request('http://example.com/a', { method: 'POST' });

    assertEqual(new Request(source, { method: 'PUT' }).method, 'PUT');
});

test('Request rejects a body on GET', () => {
    assertThrows(() => new Request('http://example.com/', { method: 'GET', body: 'x' }), 'TypeError');
});

test('Request rejects a body on HEAD', () => {
    assertThrows(() => new Request('http://example.com/', { method: 'HEAD', body: 'x' }), 'TypeError');
});

test('Request parses a urlencoded body as FormData', async () => {
    const request = new Request('http://example.com/', {
        method: 'POST',
        body: 'a=1&b=x+y',
        headers: { 'content-type': 'application/x-www-form-urlencoded' }
    });
    const form = await request.formData();

    assertEqual(form.get('a'), '1');
    assertEqual(form.get('b'), 'x y');
});

test('Request parses a multipart body as FormData', async () => {
    const boundary = '----conformance';
    const body = '--' + boundary + '\r\n'
        + 'Content-Disposition: form-data; name="a"\r\n\r\n'
        + '1\r\n'
        + '--' + boundary + '--\r\n';
    const request = new Request('http://example.com/', {
        method: 'POST',
        body: body,
        headers: { 'content-type': 'multipart/form-data; boundary=' + boundary }
    });
    const form = await request.formData();

    assertEqual(form.get('a'), '1');
});

test('Request exposes a body stream', () => {
    const request = new Request('http://example.com/', { method: 'POST', body: 'x' });

    assert(request.body instanceof ReadableStream, 'body is not a ReadableStream');
});

test('Request has a null body without one', () => {
    assertEqual(new Request('http://example.com/').body, null);
});

test('Request exposes an AbortSignal', () => {
    const request = new Request('http://example.com/');

    assert(request.signal instanceof AbortSignal, 'signal is not an AbortSignal');
});

test('Request takes a signal from init', () => {
    const controller = new AbortController();
    const request = new Request('http://example.com/', { signal: controller.signal });

    assertEqual(request.signal.aborted, false);
    controller.abort();
    assertEqual(request.signal.aborted, true);
});

test('Request defaults redirect to follow', () => {
    assertEqual(new Request('http://example.com/').redirect, 'follow');
});

test('Request defaults integrity to the empty string', () => {
    assertEqual(new Request('http://example.com/').integrity, '');
});

test('Request defaults keepalive to false', () => {
    assertEqual(new Request('http://example.com/').keepalive, false);
});
