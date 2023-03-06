import { h, Component, render } from 'https://unpkg.com/preact?module';
import htm from 'https://unpkg.com/htm?module';

const html = htm.bind(h);
function AppCPU (propsCPU, propsMemory) {
    return html`
        <div>
            <h2>CPU Usage</h2>
            ${propsCPU.cpus.map((cpu) => {
               return html`
                   <div class="bar">
                       <div class="bar-inner" style="width: ${cpu}%"></div>
                       <label>${cpu.toFixed(2)}%</label>
                   </div>
               `; 
            })} 
            <h2>Memory Usage</h2>
            <div>
                <div class="bar">
                </div>
            </div>
        </div>
    `;
}

let url = new URL("realtime/cpus", window.location.href);
url.protocol = url.protocol.replace("http", "ws");
let ws = new WebSocket(url.href);
let url2 = new URL("realtime/memory", window.location.href);
url2.protocol = url2.protocol.replace("http", "ws");
let ws2 = new WebSocket(url2.href);
let jsonCPU;
let jsonMem;
ws.onmessage = (ev) => {
    ws2.onmessage = (ev2) => {
        jsonMem = JSON.parse(ev2.data);
    };
    jsonCPU = JSON.parse(ev.data);
    render(html`<${AppCPU} cpus=${jsonCPU} memory=${jsonMem}></${AppCPU}>`, document.body);
};



