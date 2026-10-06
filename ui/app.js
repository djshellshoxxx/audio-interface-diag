const modes = {
  live: {
    title: "Live",
    subtitle: "Passive performance monitoring",
    panel: "Live health",
    button: "Start passive monitor",
  },
  daw: {
    title: "DAW",
    subtitle: "Production diagnostics without stealing interface ownership",
    panel: "DAW session health",
    button: "Select DAW session",
  },
  engineer: {
    title: "Engineer / Technician",
    subtitle: "Active qualification and bench diagnostics",
    panel: "Bench test console",
    button: "Choose test",
  },
  devices: {
    title: "Devices",
    subtitle: "Host APIs, endpoints and reported capabilities",
    panel: "Device inventory",
    button: "Refresh devices",
  },
  reports: {
    title: "Reports",
    subtitle: "Sessions, comparisons and support bundles",
    panel: "Diagnostic reports",
    button: "Open reports",
  },
};

const titleEl = document.getElementById("title");
const subtitleEl = document.getElementById("subtitle");
const panelTitleEl = document.getElementById("panel-title");
const primaryButton = document.getElementById("primary");
const healthEl = document.getElementById("health");
const elapsedEl = document.getElementById("elapsed");
const timelineEl = document.getElementById("timeline");

let startTime = null;
let timer = null;

function activeMode() {
  return document.querySelector(".nav.active").dataset.mode;
}

function addEvent(message) {
  if (timelineEl.querySelector(".muted")) {
    timelineEl.innerHTML = "";
  }

  const event = document.createElement("div");
  event.className = "event";
  event.textContent = new Date().toLocaleTimeString() + "  " + message;
  timelineEl.prepend(event);
}

function stopLiveMonitor(reason) {
  if (!timer) {
    return;
  }

  clearInterval(timer);
  timer = null;
  startTime = null;
  elapsedEl.textContent = "00:00:00";
  healthEl.textContent = "Monitoring idle";

  if (activeMode() === "live") {
    primaryButton.textContent = modes.live.button;
  }

  addEvent(reason);
}

document.querySelectorAll(".nav").forEach((button) => {
  button.addEventListener("click", () => {
    const nextMode = button.dataset.mode;

    if (timer && nextMode !== "live") {
      stopLiveMonitor("Passive monitor stopped because the view changed.");
    }

    document
      .querySelectorAll(".nav")
      .forEach((item) => item.classList.remove("active"));
    button.classList.add("active");

    const mode = modes[nextMode];
    titleEl.textContent = mode.title;
    subtitleEl.textContent = mode.subtitle;
    panelTitleEl.textContent = mode.panel;
    primaryButton.textContent = mode.button;
  });
});

primaryButton.addEventListener("click", () => {
  if (activeMode() !== "live") {
    addEvent("UI action selected; backend wiring is pending for this workflow.");
    return;
  }

  if (timer) {
    stopLiveMonitor("Passive monitor stopped.");
    return;
  }

  startTime = Date.now();
  healthEl.textContent = "Passive monitor active";
  primaryButton.textContent = "Stop monitor";
  addEvent("Passive monitoring started.");

  timer = setInterval(() => {
    const seconds = Math.floor((Date.now() - startTime) / 1000);
    const hours = String(Math.floor(seconds / 3600)).padStart(2, "0");
    const minutes = String(Math.floor((seconds % 3600) / 60)).padStart(2, "0");
    const remainder = String(seconds % 60).padStart(2, "0");

    elapsedEl.textContent = `${hours}:${minutes}:${remainder}`;
  }, 1000);
});
