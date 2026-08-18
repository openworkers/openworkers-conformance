import { error, fail } from '@sveltejs/kit';
import type { Actions, PageServerLoad } from './$types';

const ITEMS: Record<string, { name: string; price: number }> = {
  '1': { name: 'Widget', price: 9.99 },
  '2': { name: 'Gadget & "Gizmo"', price: 42 }
};

// Fixed timestamp: every byte this fixture renders has to be reproducible.
const EPOCH = 1700000000000;

export const load: PageServerLoad = ({ params, url, cookies, request }) => {
  const item = ITEMS[params.id];

  if (!item) {
    error(404, `no item ${params.id}`);
  }

  const visits = Number(cookies.get('ow_visits') ?? '0') + 1;

  cookies.set('ow_visits', String(visits), { path: '/', maxAge: 3600 });
  cookies.set('ow_last_item', params.id, { path: '/items', maxAge: 60, httpOnly: false });

  // Repeated reference: devalue emits it once and points at it twice.
  const shared = { id: params.id, name: item.name };

  return {
    item: { ...item, id: params.id },
    visits,
    query: url.searchParams.get('q'),
    tags: url.searchParams.getAll('tag'),
    accept: request.headers.get('accept'),
    // Types JSON cannot carry, so the devalue codec has to do real work.
    exotic: {
      date: new Date(EPOCH),
      map: new Map([
        ['a', 1],
        ['b', 2]
      ]),
      set: new Set([1, 2, 3]),
      big: 9007199254740993n,
      undef: undefined,
      nan: NaN,
      negZero: -0,
      inf: Infinity,
      re: /ab+c/gi,
      // e-acute, a CJK ideograph, and a rocket: 2-, 3- and 4-byte UTF-8.
      unicode: '\u00e9\u4e2d\ud83d\ude80',
      ref1: shared,
      ref2: shared
    }
  };
};

export const actions: Actions = {
  greet: async ({ request, cookies }) => {
    const form = await request.formData();
    const name = String(form.get('name') ?? '');

    if (!name) {
      return fail(400, { field: 'name', message: 'name is required' });
    }

    cookies.set('ow_greeted', name, { path: '/', maxAge: 60 });

    return { greeting: `Hello, ${name}`, at: new Date(EPOCH) };
  }
};
