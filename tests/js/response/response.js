// WinterTC Minimum Common API: Fetch Standard, the `Response` interface.

test('Response is a constructor', () => {
    assertType(Response, 'function');
});

test('Response defaults to 200', () => {
    const response = new Response();

    assertEqual(response.status, 200);
    assertEqual(response.ok, true);
});

test('Response defaults statusText to the empty string', () => {
    assertEqual(new Response().statusText, '');
});

test('Response keeps a custom statusText', () => {
    assertEqual(new Response(null, { statusText: 'Nope' }).statusText, 'Nope');
});

test('Response reports ok false for 404', () => {
    const response = new Response('missing', { status: 404 });

    assertEqual(response.status, 404);
    assertEqual(response.ok, false);
});

test('Response reads a string body as text', async () => {
    assertEqual(await new Response('hello').text(), 'hello');
});

test('Response reads an empty body as the empty string', async () => {
    assertEqual(await new Response().text(), '');
});

test('Response reads a body as json', async () => {
    const parsed = await new Response('{"a":[1,2]}').json();

    assertEqual(parsed.a[1], 2);
});

test('Response rejects invalid json', async () => {
    await assertRejects(new Response('nope').json());
});

test('Response reads a body as an ArrayBuffer', async () => {
    const buffer = await new Response('abc').arrayBuffer();

    assertEqual(buffer.byteLength, 3);
    assertEqual(new Uint8Array(buffer)[2], 99);
});

test('Response reads a typed array body', async () => {
    assertEqual(await new Response(new Uint8Array([104, 105])).text(), 'hi');
});

test('Response keeps binary bytes intact', async () => {
    const source = new Uint8Array([0, 1, 200, 255]);
    const bytes = new Uint8Array(await new Response(source).arrayBuffer());

    assertArrayEqual(bytes, [0, 1, 200, 255]);
});

test('Response takes headers from init', () => {
    const response = new Response('x', { headers: { 'X-A': '1' } });

    assertEqual(response.headers.get('x-a'), '1');
});

test('Response infers a text content type', () => {
    assertEqual(new Response('x').headers.get('content-type'), 'text/plain;charset=UTF-8');
});

test('Response infers a urlencoded content type', () => {
    const response = new Response(new URLSearchParams('a=1'));

    assertEqual(response.headers.get('content-type'), 'application/x-www-form-urlencoded;charset=UTF-8');
});

test('Response has no content type for a typed array body', () => {
    assertEqual(new Response(new Uint8Array([1])).headers.get('content-type'), null);
});

test('Response rejects a body with status 204', () => {
    assertThrows(() => new Response('x', { status: 204 }), 'TypeError');
});

test('Response rejects a body with status 304', () => {
    assertThrows(() => new Response('x', { status: 304 }), 'TypeError');
});

test('Response rejects a status below 200', () => {
    assertThrows(() => new Response(null, { status: 199 }), 'RangeError');
});

test('Response rejects a status above 599', () => {
    assertThrows(() => new Response(null, { status: 600 }), 'RangeError');
});

test('Response tracks bodyUsed', async () => {
    const response = new Response('x');

    assertEqual(response.bodyUsed, false);
    await response.text();
    assertEqual(response.bodyUsed, true);
});

test('Response refuses a second read', async () => {
    const response = new Response('x');
    await response.text();

    await assertRejects(response.text(), 'TypeError');
});

test('Response clone reads the body twice', async () => {
    const response = new Response('x');
    const copy = response.clone();

    assertEqual(await response.text(), 'x');
    assertEqual(await copy.text(), 'x');
});

test('Response exposes a body stream', () => {
    assert(new Response('x').body instanceof ReadableStream, 'body is not a ReadableStream');
});

test('Response has a null body without one', () => {
    assertEqual(new Response().body, null);
});

test('Response accepts a ReadableStream body', async () => {
    const stream = new ReadableStream({
        start(controller) {
            controller.enqueue(new TextEncoder().encode('streamed'));
            controller.close();
        }
    });

    assertEqual(await new Response(stream).text(), 'streamed');
});

test('Response.json sets the json content type', async () => {
    const response = Response.json({ a: 1 });

    assertEqual(response.headers.get('content-type'), 'application/json');
    assertEqual(await response.text(), '{"a":1}');
});

test('Response.json takes an init', () => {
    assertEqual(Response.json({}, { status: 201 }).status, 201);
});

test('Response.redirect sets status and location', () => {
    const response = Response.redirect('http://example.com/a', 302);

    assertEqual(response.status, 302);
    assertEqual(response.headers.get('location'), 'http://example.com/a');
});

test('Response.redirect defaults to 302', () => {
    assertEqual(Response.redirect('http://example.com/a').status, 302);
});

test('Response.redirect rejects a non-redirect status', () => {
    assertThrows(() => Response.redirect('http://example.com/a', 200), 'RangeError');
});

test('Response.error is a network error', () => {
    const response = Response.error();

    assertEqual(response.status, 0);
    assertEqual(response.type, 'error');
    assertEqual(response.ok, false);
});

test('Response reports a default type', () => {
    assertEqual(new Response('x').type, 'default');
});

test('Response reports an empty url', () => {
    assertEqual(new Response('x').url, '');
});

test('Response reports redirected false', () => {
    assertEqual(new Response('x').redirected, false);
});

test('Response parses a urlencoded body as FormData', async () => {
    const response = new Response('a=1&b=x+y', {
        headers: { 'content-type': 'application/x-www-form-urlencoded' }
    });
    const form = await response.formData();

    assertEqual(form.get('a'), '1');
    assertEqual(form.get('b'), 'x y');
});

test('Response reads a body as a Blob', async () => {
    const blob = await new Response('x').blob();

    assertEqual(blob.size, 1);
    assertEqual(await blob.text(), 'x');
});

test('Response reads a body as bytes', async () => {
    assertArrayEqual(await new Response('hi').bytes(), [104, 105]);
});
