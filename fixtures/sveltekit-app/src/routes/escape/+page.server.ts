import type { PageServerLoad } from './$types';

const FIXED = '<script>alert("xss & <b>")</script>';

export const load: PageServerLoad = ({ url }) => ({
  fixed: FIXED,
  supplied: url.searchParams.get('input') ?? '',
  attr: url.searchParams.get('attr') ?? 'a"b\'c<d>e&f'
});
