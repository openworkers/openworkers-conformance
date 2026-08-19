// WinterTC Minimum Common API: Web Cryptography, `crypto` and `crypto.subtle`.

function hex(buffer) {
    const bytes = new Uint8Array(buffer);
    let out = '';

    for (let i = 0; i < bytes.length; i++) {
        out += bytes[i].toString(16).padStart(2, '0');
    }

    return out;
}

function ascii(text) {
    return new TextEncoder().encode(text);
}

test('crypto is an object', () => {
    assertType(crypto, 'object');
});

test('crypto is a Crypto', () => {
    assert(crypto instanceof Crypto, 'crypto is not a Crypto');
});

test('crypto.getRandomValues is a function', () => {
    assertType(crypto.getRandomValues, 'function');
});

test('crypto.getRandomValues returns the array it was given', () => {
    const bytes = new Uint8Array(8);

    assert(crypto.getRandomValues(bytes) === bytes, 'a different object came back');
});

test('crypto.getRandomValues writes random bytes', () => {
    const bytes = new Uint8Array(64);
    crypto.getRandomValues(bytes);

    assert(Array.prototype.some.call(bytes, (b) => b !== 0), 'all 64 bytes are zero');
});

test('crypto.getRandomValues fills a Uint32Array', () => {
    const words = new Uint32Array(16);
    crypto.getRandomValues(words);

    assert(Array.prototype.some.call(words, (w) => w !== 0), 'all 16 words are zero');
});

test('crypto.getRandomValues fills an Int8Array', () => {
    assertEqual(crypto.getRandomValues(new Int8Array(4)).length, 4);
});

test('crypto.getRandomValues rejects a float array', () => {
    assertThrows(() => crypto.getRandomValues(new Float32Array(4)), 'TypeMismatchError');
});

test('crypto.getRandomValues rejects more than 65536 bytes', () => {
    assertThrows(() => crypto.getRandomValues(new Uint8Array(65537)), 'QuotaExceededError');
});

test('crypto.randomUUID returns a v4 uuid', () => {
    const uuid = crypto.randomUUID();

    assertType(uuid, 'string');
    assert(/^[0-9a-f]{8}-[0-9a-f]{4}-4[0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$/.test(uuid), 'not a v4 uuid: ' + uuid);
});

test('crypto.randomUUID differs between calls', () => {
    assert(crypto.randomUUID() !== crypto.randomUUID(), 'two calls returned the same uuid');
});

test('crypto.subtle is an object', () => {
    assertType(crypto.subtle, 'object');
});

test('crypto.subtle is a SubtleCrypto', () => {
    assert(crypto.subtle instanceof SubtleCrypto, 'subtle is not a SubtleCrypto');
});

test('crypto.subtle.digest returns an ArrayBuffer', async () => {
    const digest = await crypto.subtle.digest('SHA-256', ascii('abc'));

    assert(digest instanceof ArrayBuffer, 'digest is not an ArrayBuffer');
});

test('crypto.subtle.digest computes SHA-1', async () => {
    const digest = await crypto.subtle.digest('SHA-1', ascii('abc'));

    assertEqual(hex(digest), 'a9993e364706816aba3e25717850c26c9cd0d89d');
});

test('crypto.subtle.digest computes SHA-256', async () => {
    const digest = await crypto.subtle.digest('SHA-256', ascii('abc'));

    assertEqual(hex(digest), 'ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad');
});

test('crypto.subtle.digest computes SHA-384', async () => {
    const digest = await crypto.subtle.digest('SHA-384', ascii('abc'));

    assertEqual(hex(digest), 'cb00753f45a35e8bb5a03d699ac65007272c32ab0eded1631a8b605a43ff5bed8086072ba1e7cc2358baeca134c825a7');
});

test('crypto.subtle.digest computes SHA-512', async () => {
    const digest = await crypto.subtle.digest('SHA-512', ascii('abc'));

    assertEqual(hex(digest), 'ddaf35a193617abacc417349ae20413112e6fa4e89a97ea20a9eeee64b55d39a2192992a274fc1a836ba3c23a3feebbd454d4423643ce80e2a9ac94fa54ca49f');
});

test('crypto.subtle.digest hashes an empty input', async () => {
    const digest = await crypto.subtle.digest('SHA-256', new Uint8Array(0));

    assertEqual(hex(digest), 'e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855');
});

test('crypto.subtle.digest accepts an ArrayBuffer', async () => {
    const digest = await crypto.subtle.digest('SHA-256', ascii('abc').buffer);

    assertEqual(hex(digest), 'ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad');
});

test('crypto.subtle.digest accepts the object algorithm form', async () => {
    const digest = await crypto.subtle.digest({ name: 'SHA-256' }, ascii('abc'));

    assertEqual(hex(digest), 'ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad');
});

test('crypto.subtle.digest normalizes a lowercase algorithm', async () => {
    const digest = await crypto.subtle.digest('sha-256', ascii('abc'));

    assertEqual(hex(digest), 'ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad');
});

test('crypto.subtle.digest rejects an unknown algorithm', async () => {
    await assertRejects(crypto.subtle.digest('MD5', ascii('abc')));
});

test('crypto.subtle.importKey imports a raw HMAC key', async () => {
    const key = await crypto.subtle.importKey(
        'raw',
        ascii('key'),
        { name: 'HMAC', hash: 'SHA-256' },
        false,
        ['sign', 'verify']
    );

    assert(key instanceof CryptoKey, 'not a CryptoKey');
    assertEqual(key.type, 'secret');
});

test('crypto.subtle.sign computes an HMAC-SHA-256', async () => {
    const key = await crypto.subtle.importKey(
        'raw',
        ascii('key'),
        { name: 'HMAC', hash: 'SHA-256' },
        false,
        ['sign']
    );
    const signature = await crypto.subtle.sign('HMAC', key, ascii('The quick brown fox jumps over the lazy dog'));

    assertEqual(hex(signature), 'f7bc83f430538424b13298e6aa6fb143ef4d59a14946175997479dbc2d1a3cd8');
});

test('crypto.subtle.verify accepts a valid HMAC', async () => {
    const key = await crypto.subtle.importKey(
        'raw',
        ascii('key'),
        { name: 'HMAC', hash: 'SHA-256' },
        false,
        ['sign', 'verify']
    );
    const message = ascii('hello');
    const signature = await crypto.subtle.sign('HMAC', key, message);

    assertEqual(await crypto.subtle.verify('HMAC', key, signature, message), true);
});

test('crypto.subtle.verify rejects a tampered message', async () => {
    const key = await crypto.subtle.importKey(
        'raw',
        ascii('key'),
        { name: 'HMAC', hash: 'SHA-256' },
        false,
        ['sign', 'verify']
    );
    const signature = await crypto.subtle.sign('HMAC', key, ascii('hello'));

    assertEqual(await crypto.subtle.verify('HMAC', key, signature, ascii('hellp')), false);
});

test('crypto.subtle.generateKey builds an ECDSA pair', async () => {
    const pair = await crypto.subtle.generateKey({ name: 'ECDSA', namedCurve: 'P-256' }, true, ['sign', 'verify']);

    assertEqual(pair.privateKey.type, 'private');
    assertEqual(pair.publicKey.type, 'public');
});

test('crypto.subtle.exportKey exports a raw AES key', async () => {
    const key = await crypto.subtle.generateKey({ name: 'AES-GCM', length: 256 }, true, ['encrypt', 'decrypt']);
    const raw = await crypto.subtle.exportKey('raw', key);

    assertEqual(raw.byteLength, 32);
});

test('crypto.subtle.encrypt round-trips AES-GCM', async () => {
    const key = await crypto.subtle.generateKey({ name: 'AES-GCM', length: 256 }, true, ['encrypt', 'decrypt']);
    const iv = crypto.getRandomValues(new Uint8Array(12));
    const cipher = await crypto.subtle.encrypt({ name: 'AES-GCM', iv: iv }, key, ascii('secret'));
    const plain = await crypto.subtle.decrypt({ name: 'AES-GCM', iv: iv }, key, cipher);

    assertEqual(new TextDecoder().decode(plain), 'secret');
});
