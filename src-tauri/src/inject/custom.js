// ============================================================================
// PageSynapse (页内突触) & God Script Anti-Detection Military Rules 1~7 (POC Demo)
// Dedicated In-Page Bridge for Agentic RPA Container (NodeBox / Pake-Demo)
// ============================================================================
(() => {
  if (window.__PageSynapse_Initialized__) return;
  window.__PageSynapse_Initialized__ = true;

  console.log('[PageSynapse] Initializing 0ms In-Page Bridge & Military Rules...');

  // Setup CSP-resistant callback sender using XMLHttpRequest fallback
  window.__PageSynapseSendCallback__ = (jsonPayload) => {
    try {
      const xhr = new XMLHttpRequest();
      xhr.open('POST', 'http://127.0.0.1:39999/callback', true);
      xhr.setRequestHeader('Content-Type', 'application/json');
      xhr.send(jsonPayload);
    } catch (e) {
      console.error('[PageSynapse] Callback error:', e);
    }
  };

  // --------------------------------------------------------------------------
  // 1. Military Rule 6: Client Hints & UA Brand Cleansing (0ms Shielding)
  // --------------------------------------------------------------------------
  try {
    if (navigator.userAgentData && Array.isArray(navigator.userAgentData.brands)) {
      const cleanBrands = navigator.userAgentData.brands.filter(
        b => !b.brand.toLowerCase().includes('webview') && !b.brand.toLowerCase().includes('edge')
      );
      Object.defineProperty(navigator, 'userAgentData', {
        get: () => ({
          brands: cleanBrands,
          mobile: false,
          platform: 'Windows',
          getHighEntropyValues: (hints) => Promise.resolve({
            architecture: 'x86',
            bitness: '64',
            brands: cleanBrands,
            model: '',
            platform: 'Windows',
            platformVersion: '10.0.0',
            uaFullVersion: '124.0.6367.207'
          })
        }),
        configurable: true
      });
    }
  } catch (e) {
    console.warn('[PageSynapse] ClientHints patch warning:', e);
  }

  // --------------------------------------------------------------------------
  // 2. Military Rule 7: Eternal-Foreground Cloaking (永远在前台幻觉锁)
  // --------------------------------------------------------------------------
  try {
    Object.defineProperty(document, 'visibilityState', {
      get: () => 'visible',
      configurable: false
    });
    Object.defineProperty(document, 'hidden', {
      get: () => false,
      configurable: false
    });
    Document.prototype.hasFocus = () => true;

    const origAddEventListener = EventTarget.prototype.addEventListener;
    EventTarget.prototype.addEventListener = function(type, listener, options) {
      if (type === 'visibilitychange' || type === 'blur') {
        // Prevent background detection listeners from registering
        return;
      }
      return origAddEventListener.call(this, type, listener, options);
    };
  } catch (e) {
    console.warn('[PageSynapse] Foreground cloaking warning:', e);
  }

  // --------------------------------------------------------------------------
  // 3. PageSynapse 4-Instruction Set (Locate, Click, Write, Harvest/Verify)
  // --------------------------------------------------------------------------
  window.__PageSynapse__ = {
    // Instruction 1: Locate (空间测绘与坐标返回)
    locate: (selector) => {
      const el = document.querySelector(selector);
      if (!el) return { found: false, error: 'Element not found: ' + selector };
      const rect = el.getBoundingClientRect();
      return {
        found: true,
        x: Math.round(rect.left + rect.width / 2),
        y: Math.round(rect.top + rect.height / 2),
        width: Math.round(rect.width),
        height: Math.round(rect.height),
        visible: rect.width > 0 && rect.height > 0
      };
    },

    // Instruction 2: Click (四段式拟人动力学执行)
    click: (x, y) => {
      const el = document.elementFromPoint(x, y) || document.body;
      const now = performance.now();
      const createEvent = (type) => new MouseEvent(type, {
        bubbles: true,
        cancelable: true,
        view: window,
        clientX: x,
        clientY: y,
        screenX: x + window.screenX,
        screenY: y + window.screenY,
        button: 0,
        buttons: 1
      });

      el.dispatchEvent(createEvent('pointerdown'));
      el.dispatchEvent(createEvent('mousedown'));
      el.dispatchEvent(createEvent('pointerup'));
      el.dispatchEvent(createEvent('mouseup'));
      el.click();

      return { status: true, action: 'click', x, y, targetTag: el.tagName };
    },

    // Instruction 3: Write (闭环写入与防清空反馈)
    write: (selector, text) => {
      const el = document.querySelector(selector);
      if (!el) return { status: false, error: 'Input not found' };
      el.focus();
      el.value = text;
      el.dispatchEvent(new Event('input', { bubbles: true }));
      el.dispatchEvent(new Event('change', { bubbles: true }));
      const verified = (el.value === text);
      return { status: verified, actualValue: el.value, expectedText: text };
    },

    // Instruction 4: Harvest (SSR 初始化静态渲染树及情报抓取)
    harvest: () => {
      const ssrNext = document.getElementById('__NEXT_DATA__');
      const ssrNuxt = document.getElementById('__NUXT__');
      return {
        url: window.location.href,
        title: document.title,
        hasSSR: !!(ssrNext || ssrNuxt || window.__INITIAL_STATE__),
        ssrData: ssrNext ? JSON.parse(ssrNext.textContent) : (window.__INITIAL_STATE__ || null)
      };
    }
  };

  console.log('[PageSynapse] Node Bridge Ready! Access via window.__PageSynapse__');
})();
