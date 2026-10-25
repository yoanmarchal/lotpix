import params from '@params';

const $ = (sel, root = document) => root.querySelector(sel);
const $$ = (sel, root = document) => [...root.querySelectorAll(sel)];

// Texts in the language of the page (layouts/baseof.html), and its locale for numbers and dates
const t = JSON.parse($('#i18n')?.textContent || '{}');
const locale = document.documentElement.lang || 'en';
const fill = (text, values) => text.replace(/\{(\w+)\}/g, (_, key) => values[key] ?? '');

/* ---------- Light / dark theme ---------- */
$('[data-theme-toggle]')?.addEventListener('click', () => {
  const dark = document.documentElement.classList.toggle('dark');
  try { localStorage.setItem('theme', dark ? 'dark' : 'light'); } catch { /* storage unavailable */ }
});

/* ---------- Language: the choice is kept for the next visits (see layouts/_partials/head.html) ---------- */
$$('[data-lang]').forEach((link) => link.addEventListener('click', () => {
  try { localStorage.setItem('lang', link.dataset.lang); } catch { /* storage unavailable */ }
}));
// The menu closes when clicking elsewhere
document.addEventListener('click', (event) => {
  const menu = $('[data-language-menu]');
  if (menu?.open && !menu.contains(event.target)) menu.open = false;
});

/* ---------- Header: background while scrolling ---------- */
const header = $('[data-header]');
// A sentinel is observed rather than reading scrollY: no forced layout.
if (header && 'IntersectionObserver' in window) {
  const sentinel = document.createElement('div');
  sentinel.setAttribute('aria-hidden', 'true');
  sentinel.style.cssText = 'position:absolute;top:0;left:0;width:1px;height:8px;pointer-events:none';
  document.body.prepend(sentinel);
  new IntersectionObserver(([entry]) => {
    header.toggleAttribute('data-scrolled', !entry.isIntersecting);
  }).observe(sentinel);
}

/* ---------- Reveal while scrolling ---------- */
const revealed = $$('[data-reveal]');
if ('IntersectionObserver' in window) {
  const io = new IntersectionObserver((entries) => {
    for (const entry of entries) {
      if (!entry.isIntersecting) continue;
      entry.target.classList.add('is-visible');
      io.unobserve(entry.target);
    }
  }, { rootMargin: '0px 0px -8% 0px' });
  revealed.forEach((el) => io.observe(el));
} else {
  revealed.forEach((el) => el.classList.add('is-visible'));
}

/* ---------- Demo of the quality setting (same labels as the application) ---------- */
const demo = $('[data-quality-demo]');
if (demo) {
  const range = $('input[type=range]', demo);
  const value = $('[data-quality-value]', demo);
  const hint = $('[data-quality-hint]', demo);
  const buttons = $$('[data-format]', demo);
  let format = 'webp';

  const render = () => {
    const q = Number(range.value);
    const lossless = format === 'png';
    range.disabled = lossless;
    range.style.setProperty('--fill', `${q}%`);
    value.textContent = fill(t.percent, { value: lossless ? 100 : q });
    hint.textContent = lossless ? t.qualityLossless
      : q < 30 ? t.qualityLow
      : q < 70 ? t.qualityBalanced
      : t.qualityHigh;
  };

  range.addEventListener('input', render);
  buttons.forEach((btn) => btn.addEventListener('click', () => {
    format = btn.dataset.format;
    buttons.forEach((b) => b.setAttribute('aria-checked', String(b === btn)));
    render();
  }));
  render();
}

/* ---------- System detection ---------- */
function detectOS() {
  const platform = (navigator.userAgentData?.platform || navigator.platform || '').toLowerCase();
  const ua = navigator.userAgent.toLowerCase();
  if (/android|iphone|ipad|ipod/.test(ua)) return null;
  if (platform.includes('win') || ua.includes('windows')) return 'windows';
  if (platform.includes('linux') || ua.includes('linux')) return 'linux';
  return null;
}

const os = detectOS();
const PRIMARY = { windows: 'win-x64', linux: 'deb-x64' };
const OS_LABEL = { windows: 'Windows', linux: 'Linux' };

if (os) {
  const card = $(`[data-os-card="${os}"]`);
  card?.setAttribute('data-current', '');
  $('[data-current-badge]', card)?.classList.remove('hidden');
  const label = $('[data-primary-label]');
  if (label) label.textContent = fill(t.downloadFor, { os: OS_LABEL[os] });
}

/* ---------- Direct links to the latest GitHub release ---------- */
const formatSize = (bytes) => `${(bytes / 1024 / 1024).toLocaleString(locale, { maximumFractionDigits: 0 })} ${t.unitMb}`;

async function loadRelease() {
  const res = await fetch(`https://api.github.com/repos/${params.repo}/releases/latest`, {
    headers: { Accept: 'application/vnd.github+json' },
  });
  if (!res.ok) throw new Error(`GitHub API ${res.status}`);
  return res.json();
}

loadRelease().then((release) => {
  const version = release.tag_name.replace(/^v/, '');
  // Only rewritten if the version changed: a new text node would restart the LCP.
  $$('[data-version]').forEach((el) => { if (el.textContent !== version) el.textContent = version; });

  const date = release.published_at && new Date(release.published_at);
  const dateEl = $('[data-release-date]');
  if (date && dateEl) {
    dateEl.textContent = fill(t.published, {
      date: date.toLocaleDateString(locale, { day: 'numeric', month: 'long', year: 'numeric' }),
    });
  }

  const assetUrls = {};
  $$('[data-asset]').forEach((link) => {
    const re = new RegExp(link.dataset.match, 'i');
    const asset = release.assets.find((a) => re.test(a.name));
    if (!asset) return;
    link.href = asset.browser_download_url;
    link.title = asset.name;
    const size = $('[data-asset-size]', link);
    if (size) size.textContent = formatSize(asset.size);
    assetUrls[link.dataset.asset] = asset.browser_download_url;
  });

  // The main button downloads the right file directly when the system is recognized.
  const primary = $('[data-primary-download]');
  const url = os && assetUrls[PRIMARY[os]];
  if (primary && url) {
    primary.href = url;
    const meta = $('[data-primary-meta]');
    if (meta) {
      meta.innerHTML = '';
      meta.append(`v${version} · ${os === 'windows' ? t.windowsMeta : t.linuxMeta} · `);
      const other = Object.assign(document.createElement('a'), {
        href: '#download',
        textContent: t.otherPlatforms,
        className: 'underline decoration-zinc-400/50 underline-offset-4 hover:text-zinc-900 dark:hover:text-white',
      });
      meta.append(other);
    }
  }
}).catch((err) => {
  // The links keep pointing to the releases page.
  console.warn('GitHub release unavailable:', err);
});
