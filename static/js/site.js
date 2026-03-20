document.addEventListener("DOMContentLoaded", () => {
  initNavToggle();
  initAutoReload();
});

function initNavToggle() {
  const toggle = document.querySelector(".nav-toggle");
  const nav = document.querySelector(".site-nav");

  if (!toggle || !nav) {
    return;
  }

  toggle.addEventListener("click", () => {
    const expanded = toggle.getAttribute("aria-expanded") === "true";
    toggle.setAttribute("aria-expanded", String(!expanded));
    nav.classList.toggle("is-open", !expanded);
  });
}

function initAutoReload() {
  const enabled =
    document
      .querySelector('meta[name="app-auto-reload"]')
      ?.getAttribute("content") === "true";

  if (!enabled) {
    return;
  }

  const endpoint =
    document
      .querySelector('meta[name="app-auto-reload-endpoint"]')
      ?.getAttribute("content") || "/__dev/reload";
  const intervalRaw = document
    .querySelector('meta[name="app-auto-reload-interval-ms"]')
    ?.getAttribute("content");
  const intervalMs = Math.max(Number(intervalRaw) || 1200, 500);
  let currentInstanceId =
    document
      .querySelector('meta[name="app-instance-id"]')
      ?.getAttribute("content") || "";
  let reloadTriggered = false;

  const poll = async () => {
    if (reloadTriggered || document.visibilityState === "hidden") {
      return;
    }

    try {
      const response = await fetch(endpoint, {
        cache: "no-store",
        headers: { Accept: "application/json" },
      });

      if (!response.ok) {
        return;
      }

      const payload = await response.json();
      const nextInstanceId = typeof payload.instance_id === "string" ? payload.instance_id : "";

      if (!nextInstanceId) {
        return;
      }

      if (currentInstanceId && nextInstanceId !== currentInstanceId) {
        reloadTriggered = true;
        window.location.reload();
        return;
      }

      currentInstanceId = nextInstanceId;
    } catch (_error) {
      // The app may be rebuilding or restarting. The next successful poll will refresh the page.
    }
  };

  window.setInterval(poll, intervalMs);
  window.setTimeout(poll, intervalMs);
}
