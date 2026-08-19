// WinterTC Minimum Common API: File API `Blob`/`File` and XMLHttpRequest `FormData`.

test('Blob is a constructor', () => {
    assertType(Blob, 'function');
});

test('Blob reports its size', () => {
    assertEqual(new Blob(['hello']).size, 5);
});

test('Blob sizes a multibyte part in bytes', () => {
    assertEqual(new Blob(['\u00e9']).size, 2);
});

test('Blob defaults its type to the empty string', () => {
    assertEqual(new Blob(['x']).type, '');
});

test('Blob lowercases the type from options', () => {
    assertEqual(new Blob(['x'], { type: 'Text/Plain' }).type, 'text/plain');
});

test('Blob concatenates its parts', async () => {
    assertEqual(await new Blob(['a', 'b', 'c']).text(), 'abc');
});

test('Blob accepts a typed array part', async () => {
    assertEqual(await new Blob([new Uint8Array([104, 105])]).text(), 'hi');
});

test('Blob accepts another Blob as a part', async () => {
    assertEqual(await new Blob([new Blob(['a']), 'b']).text(), 'ab');
});

test('Blob reads as an ArrayBuffer', async () => {
    const buffer = await new Blob(['abc']).arrayBuffer();

    assertEqual(buffer.byteLength, 3);
});

test('Blob reads as bytes', async () => {
    assertArrayEqual(await new Blob(['hi']).bytes(), [104, 105]);
});

test('Blob slice returns a range', async () => {
    assertEqual(await new Blob(['abcdef']).slice(1, 3).text(), 'bc');
});

test('Blob slice counts from the end for a negative index', async () => {
    assertEqual(await new Blob(['abcdef']).slice(-2).text(), 'ef');
});

test('Blob exposes a stream', async () => {
    const stream = new Blob(['abc']).stream();

    assert(stream instanceof ReadableStream, 'stream is not a ReadableStream');
});

test('File is a constructor', () => {
    assertType(File, 'function');
});

test('File extends Blob', () => {
    assert(new File(['x'], 'a.txt') instanceof Blob, 'File is not a Blob');
});

test('File keeps its name', () => {
    assertEqual(new File(['x'], 'a.txt').name, 'a.txt');
});

test('File reports lastModified', () => {
    assertEqual(new File(['x'], 'a.txt', { lastModified: 5 }).lastModified, 5);
});

test('FormData is a constructor', () => {
    assertType(FormData, 'function');
});

test('FormData append then get', () => {
    const form = new FormData();
    form.append('a', '1');

    assertEqual(form.get('a'), '1');
});

test('FormData get returns null when absent', () => {
    assertEqual(new FormData().get('a'), null);
});

test('FormData keeps duplicates', () => {
    const form = new FormData();
    form.append('a', '1');
    form.append('a', '2');

    assertArrayEqual(form.getAll('a'), ['1', '2']);
});

test('FormData set replaces every value', () => {
    const form = new FormData();
    form.append('a', '1');
    form.append('a', '2');
    form.set('a', '3');

    assertArrayEqual(form.getAll('a'), ['3']);
});

test('FormData has and delete', () => {
    const form = new FormData();
    form.append('a', '1');

    assertEqual(form.has('a'), true);
    form.delete('a');
    assertEqual(form.has('a'), false);
});

test('FormData coerces a non-string value', () => {
    const form = new FormData();
    form.append('a', 1);

    assertEqual(form.get('a'), '1');
});

test('FormData iterates in insertion order', () => {
    const form = new FormData();
    form.append('b', '2');
    form.append('a', '1');
    const seen = [];

    for (const pair of form) {
        seen.push(pair[0] + '=' + pair[1]);
    }

    assertArrayEqual(seen, ['b=2', 'a=1']);
});

test('FormData holds a File value', () => {
    const form = new FormData();
    form.append('f', new File(['x'], 'a.txt'));

    assertEqual(form.get('f').name, 'a.txt');
});
