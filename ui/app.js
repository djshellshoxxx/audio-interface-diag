const modes={
 live:{title:"Live",subtitle:"Passive performance monitoring",panel:"Live health",button:"Start passive monitor"},
 daw:{title:"DAW",subtitle:"Production diagnostics without stealing interface ownership",panel:"DAW session health",button:"Select DAW session"},
 engineer:{title:"Engineer / Technician",subtitle:"Active qualification and bench diagnostics",panel:"Bench test console",button:"Choose test"},
 devices:{title:"Devices",subtitle:"Host APIs, endpoints and reported capabilities",panel:"Device inventory",button:"Refresh devices"},
 reports:{title:"Reports",subtitle:"Sessions, comparisons and support bundles",panel:"Diagnostic reports",button:"Open reports"}
};
document.querySelectorAll(".nav").forEach(b=>b.addEventListener("click",()=>{
 document.querySelectorAll(".nav").forEach(x=>x.classList.remove("active"));b.classList.add("active");
 const m=modes[b.dataset.mode];title.textContent=m.title;subtitle.textContent=m.subtitle;
 document.getElementById("panel-title").textContent=m.panel;primary.textContent=m.button;
}));
let start=null,timer=null;
primary.addEventListener("click",()=>{
 if(document.querySelector(".nav.active").dataset.mode!=="live"){addEvent("UI action selected; backend wiring pending for this workflow.");return;}
 if(timer){clearInterval(timer);timer=null;primary.textContent="Start passive monitor";health.textContent="Monitoring idle";addEvent("Passive monitor stopped.");return;}
 start=Date.now();health.textContent="Passive monitor active";primary.textContent="Stop monitor";addEvent("Passive monitoring started.");
 timer=setInterval(()=>{const s=Math.floor((Date.now()-start)/1000),h=String(Math.floor(s/3600)).padStart(2,"0"),m=String(Math.floor(s%3600/60)).padStart(2,"0"),q=String(s%60).padStart(2,"0");elapsed.textContent=`${h}:${m}:${q}`;},1000);
});
function addEvent(msg){const t=document.getElementById("timeline");if(t.querySelector(".muted"))t.innerHTML="";const e=document.createElement("div");e.className="event";e.textContent=new Date().toLocaleTimeString()+"  "+msg;t.prepend(e);}
