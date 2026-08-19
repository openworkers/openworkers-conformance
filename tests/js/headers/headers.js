// WinterTC Minimum Common API: Fetch Standard, the `Headers` interface.

test('Headers is a constructor', () => {
    assertType(Headers, 'function');
});

test('Headers starts empty', () => {
    assertArrayEqual(Array.from(new Headers().keys()), []);
});

test('Headers accepts a record', () => {
    assertEqual(new Headers({ 'X-A': '1' }).get('X-A'), '1');
});

test('Headers accepts a sequence of pairs', () => {
    assertEqual(new Headers([['x-a', '1']]).get('x-a'), '1');
});

test('Headers accepts another Headers', () => {
    assertEqual(new Headers(new Headers({ 'x-a': '1' })).get('x-a'), '1');
});

test('Headers get is case-insensitive', () => {
    const headers = new Headers({ 'Content-Type': 'text/plain' });

    assertEqual(headers.get('content-type'), 'text/plain');
    assertEqual(headers.get('CONTENT-TYPE'), 'text/plain');
});

test('Headers get returns null when absent', () => {
    assertEqual(new Headers().get('x-missing'), null);
});

test('Headers has is case-insensitive', () => {
    assertEqual(new Headers({ 'X-A': '1' }).has('x-a'), true);
});

test('Headers set overwrites', () => {
    const headers = new Headers({ 'x-a': '1' });
    headers.set('X-A', '2');

    assertEqual(headers.get('x-a'), '2');
});

test('Headers append joins with a comma and a space', () => {
    const headers = new Headers();
    headers.append('x-a', '1');
    headers.append('x-a', '2');

    assertEqual(headers.get('x-a'), '1, 2');
});

test('Headers append is case-insensitive', () => {
    const headers = new Headers({ 'x-a': '1' });
    headers.append('X-A', '2');

    assertEqual(headers.get('x-a'), '1, 2');
});

test('Headers delete removes the entry', () => {
    const headers = new Headers({ 'x-a': '1' });
    headers.delete('X-A');

    assertEqual(headers.has('x-a'), false);
});

test('Headers delete of an absent name is a no-op', () => {
    const headers = new Headers({ 'x-a': '1' });
    headers.delete('x-b');

    assertEqual(headers.get('x-a'), '1');
});

test('Headers lowercases names on iteration', () => {
    assertArrayEqual(Array.from(new Headers({ 'X-Ab': '1' }).keys()), ['x-ab']);
});

test('Headers iterates sorted by name', () => {
    const headers = new Headers({ 'x-c': '3', 'x-a': '1', 'x-b': '2' });

    assertArrayEqual(Array.from(headers.keys()), ['x-a', 'x-b', 'x-c']);
});

test('Headers entries yields name and value', () => {
    const first = new Headers({ 'x-a': '1' }).entries().next().value;

    assertArrayEqual(first, ['x-a', '1']);
});

test('Headers values yields values', () => {
    assertArrayEqual(Array.from(new Headers({ 'x-a': '1', 'x-b': '2' }).values()), ['1', '2']);
});

test('Headers forEach passes value, name and the headers', () => {
    const headers = new Headers({ 'x-a': '1' });
    const seen = [];
    headers.forEach((value, name, self) => seen.push(name + ':' + value + ':' + (self === headers)));

    assertArrayEqual(seen, ['x-a:1:true']);
});

test('Headers is iterable', () => {
    const seen = [];

    for (const pair of new Headers({ 'x-a': '1' })) {
        seen.push(pair[0] + '=' + pair[1]);
    }

    assertArrayEqual(seen, ['x-a=1']);
});

test('Headers trims surrounding whitespace from a value', () => {
    assertEqual(new Headers({ 'x-a': '  1  ' }).get('x-a'), '1');
});

test('Headers coerces a non-string value', () => {
    const headers = new Headers();
    headers.set('x-a', 1);

    assertEqual(headers.get('x-a'), '1');
});

test('Headers rejects an invalid name', () => {
    assertThrows(() => new Headers().set('x a', '1'), 'TypeError');
});

test('Headers rejects a value with a newline', () => {
    assertThrows(() => new Headers().set('x-a', 'a\nb'), 'TypeError');
});

test('Headers keeps two Set-Cookie values apart', () => {
    const headers = new Headers();
    headers.append('Set-Cookie', 'a=1');
    headers.append('Set-Cookie', 'b=2');

    assertArrayEqual(headers.getSetCookie(), ['a=1', 'b=2']);
});

test('Headers iterates Set-Cookie once per value', () => {
    const headers = new Headers();
    headers.append('Set-Cookie', 'a=1');
    headers.append('Set-Cookie', 'b=2');

    assertArrayEqual(Array.from(headers).map((pair) => pair[1]), ['a=1', 'b=2']);
});

test('Headers get combines Set-Cookie with a comma', () => {
    const headers = new Headers();
    headers.append('Set-Cookie', 'a=1');
    headers.append('Set-Cookie', 'b=2');

    assertEqual(headers.get('set-cookie'), 'a=1, b=2');
});

test('Headers getSetCookie is empty without a cookie', () => {
    assertArrayEqual(new Headers({ 'x-a': '1' }).getSetCookie(), []);
});

test('Headers set replaces every Set-Cookie', () => {
    const headers = new Headers();
    headers.append('Set-Cookie', 'a=1');
    headers.append('Set-Cookie', 'b=2');
    headers.set('Set-Cookie', 'c=3');

    assertArrayEqual(headers.getSetCookie(), ['c=3']);
});
