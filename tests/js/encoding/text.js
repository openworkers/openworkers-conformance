// WinterTC Minimum Common API: Encoding Standard, `TextEncoder` and `TextDecoder`.

test('TextEncoder is a constructor', () => {
    assertType(TextEncoder, 'function');
});

test('TextEncoder reports utf-8', () => {
    assertEqual(new TextEncoder().encoding, 'utf-8');
});

test('TextEncoder returns a Uint8Array', () => {
    assert(new TextEncoder().encode('a') instanceof Uint8Array, 'encode did not return a Uint8Array');
});

test('TextEncoder encodes ascii', () => {
    assertArrayEqual(new TextEncoder().encode('abc'), [97, 98, 99]);
});

test('TextEncoder encodes the empty string', () => {
    assertEqual(new TextEncoder().encode('').length, 0);
});

test('TextEncoder encodes undefined as the empty string', () => {
    assertEqual(new TextEncoder().encode().length, 0);
});

test('TextEncoder encodes a two-byte sequence', () => {
    assertArrayEqual(new TextEncoder().encode('\u00e9'), [0xc3, 0xa9]);
});

test('TextEncoder encodes a three-byte sequence', () => {
    assertArrayEqual(new TextEncoder().encode('\u20ac'), [0xe2, 0x82, 0xac]);
});

test('TextEncoder encodes a surrogate pair as four bytes', () => {
    assertArrayEqual(new TextEncoder().encode('\ud83d\ude00'), [0xf0, 0x9f, 0x98, 0x80]);
});

test('TextEncoder replaces a lone high surrogate', () => {
    assertArrayEqual(new TextEncoder().encode('\ud800'), [0xef, 0xbf, 0xbd]);
});

test('TextEncoder replaces a lone low surrogate', () => {
    assertArrayEqual(new TextEncoder().encode('\udc00'), [0xef, 0xbf, 0xbd]);
});

test('TextEncoder replaces a reversed surrogate pair', () => {
    assertArrayEqual(new TextEncoder().encode('\udc00\ud800'), [0xef, 0xbf, 0xbd, 0xef, 0xbf, 0xbd]);
});

test('TextEncoder encodeInto reports read and written', () => {
    const target = new Uint8Array(8);
    const result = new TextEncoder().encodeInto('ab', target);

    assertEqual(result.read, 2);
    assertEqual(result.written, 2);
    assertEqual(target[0], 97);
});

test('TextEncoder encodeInto stops at the buffer end', () => {
    const target = new Uint8Array(1);
    const result = new TextEncoder().encodeInto('\u00e9', target);

    assertEqual(result.read, 0);
    assertEqual(result.written, 0);
});

test('TextDecoder is a constructor', () => {
    assertType(TextDecoder, 'function');
});

test('TextDecoder defaults to utf-8', () => {
    assertEqual(new TextDecoder().encoding, 'utf-8');
});

test('TextDecoder decodes ascii', () => {
    assertEqual(new TextDecoder().decode(new Uint8Array([97, 98])), 'ab');
});

test('TextDecoder decodes no argument as the empty string', () => {
    assertEqual(new TextDecoder().decode(), '');
});

test('TextDecoder decodes a two-byte sequence', () => {
    assertEqual(new TextDecoder().decode(new Uint8Array([0xc3, 0xa9])), '\u00e9');
});

test('TextDecoder decodes a four-byte sequence', () => {
    assertEqual(new TextDecoder().decode(new Uint8Array([0xf0, 0x9f, 0x98, 0x80])), '\ud83d\ude00');
});

test('TextDecoder accepts an ArrayBuffer', () => {
    assertEqual(new TextDecoder().decode(new Uint8Array([97]).buffer), 'a');
});

test('TextDecoder accepts a subarray view', () => {
    const bytes = new Uint8Array([97, 98, 99]);

    assertEqual(new TextDecoder().decode(bytes.subarray(1, 2)), 'b');
});

test('TextDecoder replaces an invalid byte', () => {
    assertEqual(new TextDecoder().decode(new Uint8Array([0xff])), '\ufffd');
});

test('TextDecoder replaces a truncated sequence', () => {
    assertEqual(new TextDecoder().decode(new Uint8Array([0xe2, 0x82])), '\ufffd');
});

test('TextDecoder rejects invalid bytes when fatal', () => {
    const decoder = new TextDecoder('utf-8', { fatal: true });

    assertThrows(() => decoder.decode(new Uint8Array([0xff])), 'TypeError');
});

test('TextDecoder reports fatal', () => {
    assertEqual(new TextDecoder('utf-8', { fatal: true }).fatal, true);
    assertEqual(new TextDecoder().fatal, false);
});

test('TextDecoder strips a byte order mark by default', () => {
    assertEqual(new TextDecoder().decode(new Uint8Array([0xef, 0xbb, 0xbf, 97])), 'a');
});

test('TextDecoder keeps the byte order mark when told to', () => {
    const decoder = new TextDecoder('utf-8', { ignoreBOM: true });

    assertEqual(decoder.decode(new Uint8Array([0xef, 0xbb, 0xbf, 97])), '\ufeffa');
});

test('TextDecoder reports ignoreBOM', () => {
    assertEqual(new TextDecoder().ignoreBOM, false);
});

test('TextDecoder joins a sequence split across chunks', () => {
    const decoder = new TextDecoder();
    const head = decoder.decode(new Uint8Array([0xc3]), { stream: true });
    const tail = decoder.decode(new Uint8Array([0xa9]));

    assertEqual(head + tail, '\u00e9');
});

test('TextDecoder normalizes an uppercase label', () => {
    assertEqual(new TextDecoder('UTF-8').encoding, 'utf-8');
});

test('TextDecoder normalizes an alias label', () => {
    assertEqual(new TextDecoder('utf8').encoding, 'utf-8');
});

test('TextDecoder rejects an unknown label', () => {
    assertThrows(() => new TextDecoder('nope'), 'RangeError');
});

test('TextDecoder decodes windows-1252', () => {
    const decoder = new TextDecoder('windows-1252');

    assertEqual(decoder.decode(new Uint8Array([0xe9])), '\u00e9');
});

test('TextDecoder round-trips through TextEncoder', () => {
    const text = 'a\u00e9\u20ac\ud83d\ude00';

    assertEqual(new TextDecoder().decode(new TextEncoder().encode(text)), text);
});
