// WinterTC Minimum Common API: URL Standard, the `URL` interface.

test('URL is a constructor', () => {
    assertType(URL, 'function');
});

test('URL parses an absolute http url', () => {
    const url = new URL('http://example.com/a/b?c=1#d');

    assertEqual(url.protocol, 'http:');
    assertEqual(url.hostname, 'example.com');
    assertEqual(url.pathname, '/a/b');
    assertEqual(url.search, '?c=1');
    assertEqual(url.hash, '#d');
});

test('URL adds the root path to a bare origin', () => {
    assertEqual(new URL('http://example.com').href, 'http://example.com/');
});

test('URL drops the default port', () => {
    const url = new URL('http://example.com:80/');

    assertEqual(url.port, '');
    assertEqual(url.host, 'example.com');
});

test('URL keeps a non-default port', () => {
    const url = new URL('http://example.com:8080/');

    assertEqual(url.port, '8080');
    assertEqual(url.host, 'example.com:8080');
});

test('URL exposes an origin', () => {
    assertEqual(new URL('http://example.com:8080/a').origin, 'http://example.com:8080');
});

test('URL lowercases the scheme and host', () => {
    const url = new URL('HTTP://Example.COM/Path');

    assertEqual(url.protocol, 'http:');
    assertEqual(url.hostname, 'example.com');
    assertEqual(url.pathname, '/Path');
});

test('URL resolves a relative path against a base', () => {
    assertEqual(new URL('d', 'http://example.com/a/c').href, 'http://example.com/a/d');
});

test('URL resolves an absolute path against a base', () => {
    assertEqual(new URL('/b', 'http://example.com/a/c').href, 'http://example.com/b');
});

test('URL resolves against a base given as a URL', () => {
    const base = new URL('http://example.com/a/');

    assertEqual(new URL('x', base).href, 'http://example.com/a/x');
});

test('URL resolves a protocol-relative reference', () => {
    assertEqual(new URL('//other.example/x', 'https://example.com/').href, 'https://other.example/x');
});

test('URL removes dot segments', () => {
    assertEqual(new URL('http://example.com/a/b/../c').pathname, '/a/c');
});

test('URL rejects a string that is not a url', () => {
    assertThrows(() => new URL('not a url'), 'TypeError');
});

test('URL rejects a relative reference with no base', () => {
    assertThrows(() => new URL('/a/b'), 'TypeError');
});

test('URL strips surrounding whitespace', () => {
    assertEqual(new URL('  http://example.com/  ').href, 'http://example.com/');
});

test('URL percent-encodes a space in the path', () => {
    assertEqual(new URL('http://example.com/a b').pathname, '/a%20b');
});

test('URL percent-encodes a non-ascii path as utf-8', () => {
    assertEqual(new URL('http://example.com/\u00e9').pathname, '/%C3%A9');
});

test('URL punycodes a non-ascii host', () => {
    assertEqual(new URL('http://\u00e9xample.com/').hostname, 'xn--xample-9ua.com');
});

test('URL exposes credentials', () => {
    const url = new URL('http://user:pass@example.com/');

    assertEqual(url.username, 'user');
    assertEqual(url.password, 'pass');
});

test('URL parses a file url', () => {
    const url = new URL('file:///tmp/x');

    assertEqual(url.protocol, 'file:');
    assertEqual(url.hostname, '');
    assertEqual(url.pathname, '/tmp/x');
});

test('URL parses an opaque data url', () => {
    const url = new URL('data:text/plain,hi');

    assertEqual(url.protocol, 'data:');
    assertEqual(url.pathname, 'text/plain,hi');
});

test('URL toString equals href', () => {
    const url = new URL('http://example.com/a?b=1');

    assertEqual(url.toString(), url.href);
});

test('URL toJSON equals href', () => {
    const url = new URL('http://example.com/a?b=1');

    assertEqual(url.toJSON(), url.href);
});

test('URL setting protocol rewrites href', () => {
    const url = new URL('http://example.com/');
    url.protocol = 'https:';

    assertEqual(url.href, 'https://example.com/');
});

test('URL setting pathname rewrites href', () => {
    const url = new URL('http://example.com/a');
    url.pathname = '/b/c';

    assertEqual(url.href, 'http://example.com/b/c');
});

test('URL setting search adds the question mark', () => {
    const url = new URL('http://example.com/');
    url.search = 'a=1';

    assertEqual(url.search, '?a=1');
    assertEqual(url.href, 'http://example.com/?a=1');
});

test('URL setting hash adds the number sign', () => {
    const url = new URL('http://example.com/');
    url.hash = 'x';

    assertEqual(url.hash, '#x');
});

test('URL setting port rewrites host', () => {
    const url = new URL('http://example.com/');
    url.port = '8080';

    assertEqual(url.host, 'example.com:8080');
});

test('URL exposes searchParams', () => {
    const url = new URL('http://example.com/?a=1&b=2');

    assertEqual(url.searchParams.get('a'), '1');
    assertEqual(url.searchParams.get('b'), '2');
});

test('URL searchParams writes back to href', () => {
    const url = new URL('http://example.com/');
    url.searchParams.set('a', '1');

    assertEqual(url.href, 'http://example.com/?a=1');
});

test('URL keeps percent-encoding in the query', () => {
    assertEqual(new URL('http://example.com/?a=%26').search, '?a=%26');
});

test('URL.canParse reports a valid url', () => {
    assertEqual(URL.canParse('http://example.com/'), true);
    assertEqual(URL.canParse('nope'), false);
});

test('URL.parse returns null instead of throwing', () => {
    assertEqual(URL.parse('nope'), null);
    assertEqual(URL.parse('http://example.com/').href, 'http://example.com/');
});
