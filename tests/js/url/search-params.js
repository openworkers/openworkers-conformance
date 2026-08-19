// WinterTC Minimum Common API: URL Standard, the `URLSearchParams` interface.

test('URLSearchParams is a constructor', () => {
    assertType(URLSearchParams, 'function');
});

test('URLSearchParams parses a string with a leading question mark', () => {
    assertEqual(new URLSearchParams('?a=1').get('a'), '1');
});

test('URLSearchParams parses a string without a leading question mark', () => {
    assertEqual(new URLSearchParams('a=1&b=2').get('b'), '2');
});

test('URLSearchParams accepts a record', () => {
    const params = new URLSearchParams({ a: '1', b: '2' });

    assertEqual(params.get('a'), '1');
    assertEqual(params.get('b'), '2');
});

test('URLSearchParams accepts a sequence of pairs', () => {
    const params = new URLSearchParams([['a', '1'], ['a', '2']]);

    assertArrayEqual(params.getAll('a'), ['1', '2']);
});

test('URLSearchParams accepts another URLSearchParams', () => {
    const params = new URLSearchParams(new URLSearchParams('a=1'));

    assertEqual(params.get('a'), '1');
});

test('URLSearchParams accepts no argument', () => {
    assertEqual(new URLSearchParams().toString(), '');
});

test('URLSearchParams get returns the first value', () => {
    assertEqual(new URLSearchParams('a=1&a=2').get('a'), '1');
});

test('URLSearchParams get returns null when absent', () => {
    assertEqual(new URLSearchParams('a=1').get('b'), null);
});

test('URLSearchParams getAll returns every value', () => {
    assertArrayEqual(new URLSearchParams('a=1&b=2&a=3').getAll('a'), ['1', '3']);
});

test('URLSearchParams has reports presence', () => {
    const params = new URLSearchParams('a=1');

    assertEqual(params.has('a'), true);
    assertEqual(params.has('b'), false);
});

test('URLSearchParams has matches a value when given one', () => {
    const params = new URLSearchParams('a=1&a=2');

    assertEqual(params.has('a', '2'), true);
    assertEqual(params.has('a', '3'), false);
});

test('URLSearchParams set replaces every value', () => {
    const params = new URLSearchParams('a=1&a=2&b=3');
    params.set('a', '9');

    assertEqual(params.toString(), 'a=9&b=3');
});

test('URLSearchParams set appends when absent', () => {
    const params = new URLSearchParams('a=1');
    params.set('b', '2');

    assertEqual(params.toString(), 'a=1&b=2');
});

test('URLSearchParams append keeps duplicates', () => {
    const params = new URLSearchParams('a=1');
    params.append('a', '2');

    assertEqual(params.toString(), 'a=1&a=2');
});

test('URLSearchParams delete removes every value', () => {
    const params = new URLSearchParams('a=1&a=2&b=3');
    params.delete('a');

    assertEqual(params.toString(), 'b=3');
});

test('URLSearchParams delete accepts a value', () => {
    const params = new URLSearchParams('a=1&a=2');
    params.delete('a', '1');

    assertEqual(params.toString(), 'a=2');
});

test('URLSearchParams serializes a space as a plus', () => {
    const params = new URLSearchParams();
    params.set('a', 'x y');

    assertEqual(params.toString(), 'a=x+y');
});

test('URLSearchParams decodes a plus as a space', () => {
    assertEqual(new URLSearchParams('a=x+y').get('a'), 'x y');
});

test('URLSearchParams decodes percent escapes', () => {
    assertEqual(new URLSearchParams('a=%26%3D').get('a'), '&=');
});

test('URLSearchParams percent-encodes reserved characters', () => {
    const params = new URLSearchParams();
    params.set('a', '&=');

    assertEqual(params.toString(), 'a=%26%3D');
});

test('URLSearchParams encodes non-ascii as utf-8', () => {
    const params = new URLSearchParams();
    params.set('a', '\u00e9');

    assertEqual(params.toString(), 'a=%C3%A9');
});

test('URLSearchParams decodes non-ascii as utf-8', () => {
    assertEqual(new URLSearchParams('a=%C3%A9').get('a'), '\u00e9');
});

test('URLSearchParams reads a pair with an empty value', () => {
    assertEqual(new URLSearchParams('a=').get('a'), '');
});

test('URLSearchParams reads a pair with no equals sign', () => {
    assertEqual(new URLSearchParams('a').get('a'), '');
});

test('URLSearchParams skips empty pairs', () => {
    assertEqual(new URLSearchParams('a=1&&b=2').toString(), 'a=1&b=2');
});

test('URLSearchParams iterates in insertion order', () => {
    const params = new URLSearchParams('b=2&a=1');
    const seen = [];

    for (const pair of params) {
        seen.push(pair[0] + '=' + pair[1]);
    }

    assertArrayEqual(seen, ['b=2', 'a=1']);
});

test('URLSearchParams entries yields pairs', () => {
    const first = new URLSearchParams('a=1').entries().next().value;

    assertArrayEqual(first, ['a', '1']);
});

test('URLSearchParams keys yields names', () => {
    assertArrayEqual(Array.from(new URLSearchParams('a=1&b=2').keys()), ['a', 'b']);
});

test('URLSearchParams values yields values', () => {
    assertArrayEqual(Array.from(new URLSearchParams('a=1&b=2').values()), ['1', '2']);
});

test('URLSearchParams forEach passes value then key', () => {
    const seen = [];
    new URLSearchParams('a=1').forEach((value, key) => seen.push(key + ':' + value));

    assertArrayEqual(seen, ['a:1']);
});

test('URLSearchParams sort orders by name', () => {
    const params = new URLSearchParams('c=3&a=1&b=2');
    params.sort();

    assertEqual(params.toString(), 'a=1&b=2&c=3');
});

test('URLSearchParams sort keeps duplicate order', () => {
    const params = new URLSearchParams('b=2&a=2&a=1');
    params.sort();

    assertEqual(params.toString(), 'a=2&a=1&b=2');
});

test('URLSearchParams exposes size', () => {
    assertEqual(new URLSearchParams('a=1&a=2&b=3').size, 3);
});

test('URLSearchParams coerces non-string arguments', () => {
    const params = new URLSearchParams();
    params.set('a', 1);

    assertEqual(params.get('a'), '1');
});
