// WinterTC Minimum Common API: HTML Standard, `atob` and `btoa`.

test('btoa is a function', () => {
    assertType(btoa, 'function');
});

test('atob is a function', () => {
    assertType(atob, 'function');
});

test('btoa encodes ascii', () => {
    assertEqual(btoa('hello'), 'aGVsbG8=');
});

test('btoa encodes the empty string', () => {
    assertEqual(btoa(''), '');
});

test('btoa pads one leftover byte', () => {
    assertEqual(btoa('a'), 'YQ==');
});

test('btoa pads two leftover bytes', () => {
    assertEqual(btoa('ab'), 'YWI=');
});

test('btoa needs no padding for three bytes', () => {
    assertEqual(btoa('abc'), 'YWJj');
});

test('btoa encodes the high byte range', () => {
    assertEqual(btoa('\u00ff\u00fe'), '//4=');
});

test('btoa rejects a code point above 255', () => {
    assertThrows(() => btoa('\u00e9\u20ac'), 'InvalidCharacterError');
});

test('btoa coerces a non-string argument', () => {
    assertEqual(btoa(12), 'MTI=');
});

test('atob decodes ascii', () => {
    assertEqual(atob('aGVsbG8='), 'hello');
});

test('atob decodes the empty string', () => {
    assertEqual(atob(''), '');
});

test('atob decodes without padding', () => {
    assertEqual(atob('YQ'), 'a');
});

test('atob ignores whitespace', () => {
    assertEqual(atob(' a G V s b G 8 = '), 'hello');
});

test('atob decodes the high byte range', () => {
    const decoded = atob('//4=');

    assertEqual(decoded.charCodeAt(0), 255);
    assertEqual(decoded.charCodeAt(1), 254);
});

test('atob rejects a character outside the alphabet', () => {
    assertThrows(() => atob('a*b='), 'InvalidCharacterError');
});

test('atob rejects a length of one modulo four', () => {
    assertThrows(() => atob('aGVsbG8=a'), 'InvalidCharacterError');
});

test('atob rejects misplaced padding', () => {
    assertThrows(() => atob('a=aa'), 'InvalidCharacterError');
});

test('atob throws a DOMException', () => {
    let error;

    try {
        atob('*');
    } catch (e) {
        error = e;
    }

    assert(error instanceof DOMException, 'not a DOMException');
    assertEqual(error.name, 'InvalidCharacterError');
});

test('atob round-trips every byte value', () => {
    let binary = '';

    for (let i = 0; i < 256; i++) {
        binary += String.fromCharCode(i);
    }

    assertEqual(atob(btoa(binary)), binary);
});
