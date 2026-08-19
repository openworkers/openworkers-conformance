// WinterTC Minimum Common API: Streams Standard and the Compression Standard.

async function readAll(stream) {
    const reader = stream.getReader();
    const chunks = [];

    for (;;) {
        const result = await reader.read();

        if (result.done) {
            return chunks;
        }

        chunks.push(result.value);
    }
}

async function readText(stream) {
    const chunks = await readAll(stream);
    const decoder = new TextDecoder();
    let text = '';

    for (let i = 0; i < chunks.length; i++) {
        const chunk = chunks[i];

        text += typeof chunk === 'string' ? chunk : decoder.decode(chunk, { stream: true });
    }

    return text + decoder.decode();
}

function streamOf(values) {
    return new ReadableStream({
        start(controller) {
            for (let i = 0; i < values.length; i++) {
                controller.enqueue(values[i]);
            }

            controller.close();
        }
    });
}

test('ReadableStream is a constructor', () => {
    assertType(ReadableStream, 'function');
});

test('ReadableStream yields the chunks it was given', async () => {
    assertArrayEqual(await readAll(streamOf(['a', 'b'])), ['a', 'b']);
});

test('ReadableStream reports done after close', async () => {
    const reader = streamOf([]).getReader();
    const result = await reader.read();

    assertEqual(result.done, true);
    assertEqual(result.value, undefined);
});

test('ReadableStream calls pull for each read', async () => {
    let pulls = 0;
    const stream = new ReadableStream({
        pull(controller) {
            pulls++;

            if (pulls > 2) {
                controller.close();
                return;
            }

            controller.enqueue(pulls);
        }
    });

    assertArrayEqual(await readAll(stream), [1, 2]);
});

test('ReadableStream reports locked while a reader is held', () => {
    const stream = streamOf(['a']);

    assertEqual(stream.locked, false);
    stream.getReader();
    assertEqual(stream.locked, true);
});

test('ReadableStream refuses a second reader', () => {
    const stream = streamOf(['a']);
    stream.getReader();

    assertThrows(() => stream.getReader(), 'TypeError');
});

test('ReadableStream releaseLock unlocks the stream', () => {
    const stream = streamOf(['a']);
    const reader = stream.getReader();
    reader.releaseLock();

    assertEqual(stream.locked, false);
});

test('ReadableStream cancel runs the cancel callback', async () => {
    let reason = null;
    const stream = new ReadableStream({
        cancel(given) { reason = given; }
    });
    await stream.cancel('done');

    assertEqual(reason, 'done');
});

test('ReadableStream surfaces a controller error', async () => {
    const stream = new ReadableStream({
        start(controller) { controller.error(new Error('boom')); }
    });

    await assertRejects(stream.getReader().read(), 'boom');
});

test('ReadableStream tee splits the chunks', async () => {
    const branches = streamOf(['a', 'b']).tee();

    assertArrayEqual(await readAll(branches[0]), ['a', 'b']);
    assertArrayEqual(await readAll(branches[1]), ['a', 'b']);
});

test('ReadableStream is async iterable', () => {
    assertType(streamOf(['a'])[Symbol.asyncIterator], 'function');
});

test('ReadableStream.from wraps an iterable', async () => {
    assertArrayEqual(await readAll(ReadableStream.from(['a', 'b'])), ['a', 'b']);
});

test('ReadableStream supports a byte stream', async () => {
    const stream = new ReadableStream({
        type: 'bytes',
        start(controller) {
            controller.enqueue(new Uint8Array([1, 2]));
            controller.close();
        }
    });
    const chunks = await readAll(stream);

    assertArrayEqual(chunks[0], [1, 2]);
});

test('WritableStream is a constructor', () => {
    assertType(WritableStream, 'function');
});

test('WritableStream receives what is written', async () => {
    const written = [];
    const stream = new WritableStream({
        write(chunk) { written.push(chunk); }
    });
    const writer = stream.getWriter();
    await writer.write('a');
    await writer.close();

    assertArrayEqual(written, ['a']);
});

test('TransformStream is a constructor', () => {
    assertType(TransformStream, 'function');
});

test('TransformStream maps chunks through pipeThrough', async () => {
    const upper = new TransformStream({
        transform(chunk, controller) { controller.enqueue(chunk.toUpperCase()); }
    });

    assertArrayEqual(await readAll(streamOf(['a', 'b']).pipeThrough(upper)), ['A', 'B']);
});

test('ReadableStream pipeTo drains into a writable', async () => {
    const written = [];
    await streamOf(['a', 'b']).pipeTo(new WritableStream({
        write(chunk) { written.push(chunk); }
    }));

    assertArrayEqual(written, ['a', 'b']);
});

test('CountQueuingStrategy reports a size of one', () => {
    const strategy = new CountQueuingStrategy({ highWaterMark: 3 });

    assertEqual(strategy.highWaterMark, 3);
    assertEqual(strategy.size('anything'), 1);
});

test('ByteLengthQueuingStrategy sizes by byteLength', () => {
    const strategy = new ByteLengthQueuingStrategy({ highWaterMark: 8 });

    assertEqual(strategy.size(new Uint8Array(4)), 4);
});

test('TextEncoderStream encodes a piped string', async () => {
    const chunks = await readAll(streamOf(['hi']).pipeThrough(new TextEncoderStream()));

    assertArrayEqual(chunks[0], [104, 105]);
});

test('TextDecoderStream decodes piped bytes', async () => {
    const source = streamOf([new Uint8Array([0xc3]), new Uint8Array([0xa9])]);

    assertEqual(await readText(source.pipeThrough(new TextDecoderStream())), '\u00e9');
});

test('CompressionStream round-trips through DecompressionStream', async () => {
    const compressed = streamOf([new TextEncoder().encode('hello')])
        .pipeThrough(new CompressionStream('gzip'))
        .pipeThrough(new DecompressionStream('gzip'));

    assertEqual(await readText(compressed), 'hello');
});

test('a Response body is a readable stream', async () => {
    assertEqual(await readText(new Response('streamed').body), 'streamed');
});

test('a Request body is a readable stream', async () => {
    const request = new Request('http://example.com/', { method: 'POST', body: 'sent' });

    assertEqual(await readText(request.body), 'sent');
});
